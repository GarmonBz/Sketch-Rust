//! Sketch transfer through the locally installed SFREP COM server.
use std::{mem::ManuallyDrop, ptr};

use iced::widget::image::Handle;
use image::{codecs::png::PngEncoder, ExtendedColorType, ImageEncoder};
use windows::{
    core::{w, IUnknown, Interface, BSTR, GUID, PCWSTR},
    Win32::{
        Foundation::{E_NOINTERFACE, REGDB_E_CLASSNOTREG},
        System::{
            Com::{
                CoCreateInstance, CoInitializeEx, CoUninitialize, IDispatch, CLSCTX_INPROC_SERVER,
                COINIT_APARTMENTTHREADED, DISPATCH_METHOD, DISPPARAMS, EXCEPINFO,
            },
            Ole::{SafeArrayAccessData, SafeArrayCreateVector, SafeArrayUnaccessData},
            Variant::{
                VariantClear, VARIANT, VARIANT_0_0, VT_ARRAY, VT_BOOL, VT_BSTR, VT_DISPATCH,
                VT_EMPTY, VT_I4, VT_NULL, VT_UI1, VT_UNKNOWN,
            },
        },
    },
};

// IntegrationCom and IIntegrationCom2, as published in SFREP's COM reference.
// The installed type library declares IIntegrationCom2 as a dual interface.
const INTEGRATION_COM: GUID = GUID::from_u128(0x83cb43e0_8001_4aa0_a57f_6b8b5fe0ed2a);
const INTEGRATION_COM_2: GUID = GUID::from_u128(0x05083112_becd_429b_ab12_da5cd3f6ff48);
// The URAR's report-level Additional Sketch(es) field from SFREP's Field Details panel.
const SKETCH_IMAGE_LIST: &str = "SketchImageList";

type Result<T> = std::result::Result<T, String>;

#[derive(Debug, Clone)]
pub enum ReportStatus {
    Cancelled,
    NoReport,
    Open(Report),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Report {
    process_id: i32,
    path: String,
    primary_form: String,
    dynamic: bool,
    read_only: bool,
}

impl Report {
    pub(crate) fn transfer_availability(&self) -> Result<()> {
        if !self.dynamic {
            return Err("Sketch transfer requires a dynamic UAD 3.6 report.".into());
        }
        if self.read_only {
            return Err("Open an editable copy of this report to transfer a sketch.".into());
        }
        if self.path.is_empty() {
            return Err("Save the report in Appraise-It Pro, then check it again.".into());
        }
        Ok(())
    }

    pub(crate) fn summary(&self) -> String {
        let path = if self.path.is_empty() {
            "Unsaved report"
        } else {
            &self.path
        };
        let report_type = if self.dynamic {
            "Dynamic UAD 3.6"
        } else {
            "Legacy"
        };
        let access = if self.read_only {
            "Read-only"
        } else {
            "Editable"
        };
        format!(
            "{path}\n{report_type} · {access} · Form {}",
            self.primary_form
        )
    }
}

impl ReportStatus {
    pub(crate) fn summary(&self) -> String {
        match self {
            Self::Cancelled => "Connection cancelled.".into(),
            Self::NoReport => {
                "Connected. Open a report in Appraise-It Pro, then check again.".into()
            }
            Self::Open(report) => report.summary(),
        }
    }
}

pub(crate) async fn check_report() -> Result<ReportStatus> {
    on_worker(inspect_report).await
}

pub(crate) async fn send_sketch(target: Report, images: Vec<(usize, Handle)>) -> Result<()> {
    on_worker(move || transfer_sketch(&target, &images)).await
}

pub(crate) async fn render_pages(pages: Vec<(usize, String)>) -> Result<Vec<(usize, Handle)>> {
    on_worker(move || {
        pages
            .into_iter()
            .map(|(number, svg)| {
                let sketch = crate::report_sketch::raster(&svg)
                    .ok_or_else(|| format!("Could not render sketch page {number}."))?;
                let mut png = Vec::new();
                PngEncoder::new(&mut png)
                    .write_image(
                        &sketch.rgb,
                        sketch.width,
                        sketch.height,
                        ExtendedColorType::Rgb8,
                    )
                    .map_err(|error| format!("Could not encode sketch page {number}. {error}"))?;
                Ok((number, Handle::from_bytes(png)))
            })
            .collect()
    })
    .await
}

async fn on_worker<T: Send + 'static>(
    work: impl FnOnce() -> Result<T> + Send + 'static,
) -> Result<T> {
    let (sender, receiver) = iced::futures::channel::oneshot::channel();
    // The SFREP session picker and transaction may block. All COM calls and
    // releases stay on this single-threaded apartment, off the editor thread.
    std::thread::Builder::new()
        .name("appraise-it".into())
        .spawn(move || {
            let _ = sender.send(work());
        })
        .map_err(|error| format!("Could not start the Appraise-It Pro connection. {error}"))?;
    receiver
        .await
        .map_err(|_| "The Appraise-It Pro connection ended unexpectedly.".to_owned())?
}

struct Apartment;

impl Apartment {
    fn new() -> Result<Self> {
        // SAFETY: this thread owns the apartment and balances successful initialization in Drop.
        unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED).ok() }
            .map_err(|error| format!("Could not initialize Windows COM. {error}"))?;
        Ok(Self)
    }
}

impl Drop for Apartment {
    fn drop(&mut self) {
        // SAFETY: the apartment was successfully initialized on this thread.
        unsafe { CoUninitialize() };
    }
}

#[repr(transparent)]
struct Value(VARIANT);

impl Value {
    fn from_text(text: &str) -> Self {
        let mut value = Self(VARIANT::default());
        // SAFETY: initialize a VT_BSTR variant and transfer ownership to Value.
        unsafe {
            let data = &mut *value.0.Anonymous.Anonymous;
            data.vt = VT_BSTR;
            data.Anonymous.bstrVal = ManuallyDrop::new(BSTR::from(text));
        }
        value
    }

    fn from_i32(number: i32) -> Self {
        let mut value = Self(VARIANT::default());
        // SAFETY: initialize the union with the field identified by VT_I4.
        unsafe {
            let data = &mut *value.0.Anonymous.Anonymous;
            data.vt = VT_I4;
            data.Anonymous.lVal = number;
        }
        value
    }

    fn from_dispatch(dispatch: IDispatch) -> Self {
        let mut value = Self(VARIANT::default());
        // SAFETY: transfer the owned interface reference into the VT_DISPATCH variant.
        unsafe {
            let data = &mut *value.0.Anonymous.Anonymous;
            data.vt = VT_DISPATCH;
            data.Anonymous.pdispVal = ManuallyDrop::new(Some(dispatch));
        }
        value
    }

    fn image(bytes: &[u8]) -> Result<Self> {
        let len = u32::try_from(bytes.len()).map_err(|_| "The sketch image is too large.")?;
        if len == 0 {
            return Err("The sketch image is empty.".into());
        }
        // SAFETY: allocate a one-dimensional array of bytes for Automation.
        let array = unsafe { SafeArrayCreateVector(VT_UI1, 0, len) };
        if array.is_null() {
            return Err("Could not allocate the sketch payload for Appraise-It Pro.".into());
        }
        let mut value = Self(VARIANT::default());
        // SAFETY: the variant owns the newly allocated array and frees it in Drop.
        unsafe {
            let data = &mut *value.0.Anonymous.Anonymous;
            data.vt = VT_ARRAY | VT_UI1;
            data.Anonymous.parray = array;
        }
        let mut data = ptr::null_mut();
        // SAFETY: the array holds exactly bytes.len() bytes. Unlock before Value drops.
        unsafe {
            SafeArrayAccessData(array, &mut data)
                .map_err(|error| format!("Could not access the sketch image buffer. {error}"))?;
            ptr::copy_nonoverlapping(bytes.as_ptr(), data.cast(), bytes.len());
            SafeArrayUnaccessData(array)
                .map_err(|error| format!("Could not unlock the sketch image buffer. {error}"))?;
        }
        Ok(value)
    }

    fn data(&self) -> &VARIANT_0_0 {
        // SAFETY: the result is initialized as VT_EMPTY and populated by IDispatch.
        // The caller checks the tag before accessing the associated union field.
        unsafe { &self.0.Anonymous.Anonymous }
    }

    fn boolean(&self, method: &str) -> Result<bool> {
        let value = self.data();
        if value.vt != VT_BOOL {
            return Err(format!(
                "Appraise-It Pro returned an unexpected value for {method}."
            ));
        }
        // SAFETY: VT_BOOL identifies boolVal.
        Ok(unsafe { value.Anonymous.boolVal.0 != 0 })
    }

    fn integer(&self, method: &str) -> Result<i32> {
        let value = self.data();
        if value.vt != VT_I4 {
            return Err(format!(
                "Appraise-It Pro returned an unexpected value for {method}."
            ));
        }
        // SAFETY: VT_I4 identifies lVal.
        Ok(unsafe { value.Anonymous.lVal })
    }

    fn object(&self, label: &str) -> Result<Option<ComObject>> {
        let value = self.data();
        // SAFETY: the tag selects the matching union field. Clone or query
        // the interface to retain a reference after the variant drops.
        let dispatch = unsafe {
            match value.vt {
                VT_DISPATCH => (*value.Anonymous.pdispVal).clone(),
                VT_UNKNOWN => value
                    .Anonymous
                    .punkVal
                    .as_ref()
                    .map(Interface::cast::<IDispatch>)
                    .transpose()
                    .map_err(|error| format!("Could not access {label}. {error}"))?,
                VT_EMPTY | VT_NULL => None,
                _ => {
                    return Err(format!(
                        "Appraise-It Pro returned an unexpected value for {label}."
                    ))
                }
            }
        };
        Ok(dispatch.map(ComObject))
    }

    fn required_object(&self, label: &str) -> Result<ComObject> {
        self.object(label)?
            .ok_or_else(|| format!("Appraise-It Pro did not return {label}."))
    }

    fn text(&self, method: &str) -> Result<String> {
        let value = self.data();
        if value.vt != VT_BSTR {
            return Err(format!(
                "Appraise-It Pro returned an unexpected value for {method}."
            ));
        }
        // SAFETY: VT_BSTR identifies bstrVal, which remains owned by the variant.
        Ok(unsafe { value.Anonymous.bstrVal.to_string() })
    }
}

impl Drop for Value {
    fn drop(&mut self) {
        // SAFETY: this value owns the initialized Automation variant.
        let _ = unsafe { VariantClear(&mut self.0) };
    }
}

struct Session(IDispatch);

struct ComObject(IDispatch);

impl ComObject {
    fn call(&self, name: PCWSTR, label: &str, args: &mut [Value]) -> Result<Value> {
        invoke(&self.0, name, label, args)
    }
}

impl Drop for ComObject {
    fn drop(&mut self) {
        let _ = self.call(w!("Dispose"), "releasing report data", &mut []);
    }
}

impl Session {
    fn new() -> Result<Self> {
        // SAFETY: COM is initialized. The requested class is SFREP's installed,
        // registered IntegrationCom server and the result is an owned interface.
        let unknown: IUnknown =
            unsafe { CoCreateInstance(&INTEGRATION_COM, None, CLSCTX_INPROC_SERVER) }.map_err(
                |error| {
                    if error.code() == REGDB_E_CLASSNOTREG {
                        "Appraise-It Pro's COM integration is unavailable. Install or repair Appraise-It Pro for this Windows architecture.".into()
                    } else {
                        format!("Could not start the Appraise-It Pro COM server. {error}")
                    }
                },
            )?;
        let mut interface = ptr::null_mut();
        // SAFETY: IIntegrationCom2 is dual and starts with the IDispatch
        // vtable. QueryInterface returns an owned reference for this IID.
        unsafe {
            unknown
                .query(&INTEGRATION_COM_2, &mut interface)
                .ok()
                .map_err(|error| {
                    if error.code() == E_NOINTERFACE {
                        "Update Appraise-It Pro to version 3.5 or later to use this connection."
                            .into()
                    } else {
                        format!("Could not get the Appraise-It Pro COM interface. {error}")
                    }
                })?;
            Ok(Self(IDispatch::from_raw(interface)))
        }
    }

    fn call(&self, name: PCWSTR, label: &str, args: &mut [Value]) -> Result<Value> {
        invoke(&self.0, name, label, args)
    }

    fn initialize(&self) -> Result<bool> {
        self.call(
            w!("InitializeWithPrompt"),
            "session selection",
            &mut [Value::from_text("Sketch Rust")],
        )?
        .boolean("InitializeWithPrompt")
    }

    fn initialize_for_process(&self, process_id: i32) -> Result<()> {
        self.call(
            w!("InitializeForProcess"),
            "connecting to the reviewed session",
            &mut [Value::from_i32(process_id), Value::from_text("Sketch Rust")],
        )?;
        Ok(())
    }

    fn running(&self) -> Result<bool> {
        self.call(w!("IsRunning"), "checking the session", &mut [])?
            .boolean("IsRunning")
    }

    fn report(&self) -> Result<Option<Report>> {
        if !self
            .call(w!("HasReport"), "checking for an open report", &mut [])?
            .boolean("HasReport")?
        {
            return Ok(None);
        }
        Ok(Some(Report {
            process_id: self
                .call(w!("ProcessId"), "identifying the session", &mut [])?
                .integer("ProcessId")?,
            path: self
                .call(w!("ReportPath"), "reading the report path", &mut [])?
                .text("ReportPath")?,
            primary_form: self
                .call(w!("PrimaryFormId"), "reading the primary form", &mut [])?
                .text("PrimaryFormId")?,
            dynamic: self
                .call(w!("IsDynamicReport"), "checking the report type", &mut [])?
                .boolean("IsDynamicReport")?,
            read_only: self
                .call(w!("ReportIsReadOnly"), "checking report access", &mut [])?
                .boolean("ReportIsReadOnly")?,
        }))
    }
}

// Automation places positional arguments in reverse order.
fn invoke(
    dispatch: &IDispatch,
    name: PCWSTR,
    label: &str,
    arguments: &mut [Value],
) -> Result<Value> {
    let iid_null = GUID::zeroed();
    let mut member = 0;
    let args = DISPPARAMS {
        rgvarg: arguments.as_mut_ptr().cast(),
        cArgs: arguments.len() as u32,
        ..Default::default()
    };
    let mut value = Value(VARIANT::default());
    let mut info = EXCEPINFO::default();
    // SAFETY: name is NUL-terminated. The argument, if present, is valid
    // for the call and all output buffers are initialized and writable.
    let result = unsafe {
        dispatch
            .GetIDsOfNames(&iid_null, &name, 1, 0, &mut member)
            .and_then(|_| {
                dispatch.Invoke(
                    member,
                    &iid_null,
                    0,
                    DISPATCH_METHOD,
                    &args,
                    Some(&mut value.0),
                    Some(&mut info),
                    None,
                )
            })
    };
    // SAFETY: Automation gives the caller ownership of EXCEPINFO's BSTRs.
    unsafe {
        if let Some(fill) = info.pfnDeferredFillIn {
            let _ = fill(&mut info);
        }
        let description = ManuallyDrop::take(&mut info.bstrDescription);
        ManuallyDrop::drop(&mut info.bstrSource);
        ManuallyDrop::drop(&mut info.bstrHelpFile);
        result.map_err(|error| {
            let detail = if description.is_empty() {
                error.to_string()
            } else {
                description.to_string()
            };
            format!("Appraise-It Pro could not complete {label}. {detail}")
        })?;
    }
    Ok(value)
}

impl Drop for Session {
    fn drop(&mut self) {
        let _ = self.call(w!("Close"), "closing the connection", &mut []);
    }
}

struct Transaction<'a>(&'a Session);

impl<'a> Transaction<'a> {
    fn begin(session: &'a Session) -> Result<Self> {
        session.call(w!("BeginTransaction"), "opening a transaction", &mut [])?;
        Ok(Self(session))
    }
}

impl Drop for Transaction<'_> {
    fn drop(&mut self) {
        let _ = self
            .0
            .call(w!("EndTransaction"), "ending the transaction", &mut []);
    }
}

fn inspect_report() -> Result<ReportStatus> {
    let _apartment = Apartment::new()?;
    let session = Session::new()?;
    if !session.initialize()? {
        return Ok(ReportStatus::Cancelled);
    }
    if !session.running()? {
        return Err("Appraise-It Pro closed before its report could be checked.".into());
    }
    let _transaction = Transaction::begin(&session)?;
    Ok(session
        .report()?
        .map_or(ReportStatus::NoReport, ReportStatus::Open))
}

fn transfer_sketch(target: &Report, images: &[(usize, Handle)]) -> Result<()> {
    target.transfer_availability()?;
    if images.is_empty() {
        return Err("There are no sketch images to transfer.".into());
    }
    let _apartment = Apartment::new()?;
    let pages = images
        .iter()
        .map(|(number, handle)| {
            let Handle::Bytes(_, bytes) = handle else {
                return Err(format!("Sketch page {number} has no encoded image."));
            };
            Ok((*number, Value::image(bytes)?))
        })
        .collect::<Result<Vec<_>>>()?;
    let session = Session::new()?;
    session.initialize_for_process(target.process_id)?;
    if !session.running()? {
        return Err(
            "The reviewed Appraise-It Pro session is closed. Check the open report again.".into(),
        );
    }
    let _transaction = Transaction::begin(&session)?;
    if session.report()?.as_ref() != Some(target) {
        return Err(
            "The reviewed report has changed or closed. Check the open report again.".into(),
        );
    }
    let store = session
        .call(w!("GetDataStore"), "opening the report data store", &mut [])?
        .required_object("the report data store")?;
    let report = store
        .call(w!("GetReportFields"), "reading the subject", &mut [])?
        .required_object("the subject")?;
    let fields = report
        .call(w!("GetFields"), "reading the subject fields", &mut [])?
        .required_object("the subject fields")?;
    let sketches = fields
        .call(
            w!("GetImageListField"),
            "finding Additional Sketches",
            &mut [
                Value::from_text("None"),
                Value::from_text(SKETCH_IMAGE_LIST),
            ],
        )?
        .object("the Additional Sketches field")?
        .ok_or("This report has no Additional Sketches field.")?;
    append_sketches(&store, &sketches, pages)
}

fn append_sketches(
    store: &ComObject,
    sketches: &ComObject,
    pages: Vec<(usize, Value)>,
) -> Result<()> {
    let count = || {
        sketches
            .call(w!("DataItemCount"), "counting sketch images", &mut [])?
            .integer("DataItemCount")
    };
    let original_count = count()?;
    let mut loaded = Vec::with_capacity(pages.len());
    let result = (|| {
        // Load all images before appending any exhibits.
        for (number, png) in pages {
            let image = store
                .call(
                    w!("LoadImageData"),
                    "loading the sketch image",
                    &mut [Value::from_text(".png"), png],
                )?
                .required_object("the loaded sketch image")?;
            loaded.push((number, image));
        }
        for (number, image) in &loaded {
            sketches
                .call(
                    w!("AddDataItem"),
                    "appending the sketch image",
                    &mut [
                        Value::from_dispatch(image.0.clone()),
                        Value::from_text("FullPage"),
                        Value::from_text(&format!("Sketch Rust · page {number}")),
                        Value::from_text("SubjectPropertyImprovementSketch"),
                    ],
                )?
                .required_object("the appended sketch image")?;
        }
        Ok(())
    })();
    if let Err(error) = result {
        // SFREP's integration transaction serializes access. Roll back
        // appended exhibits explicitly and leave the pre-existing entries alone.
        let rollback = count().and_then(|current| {
            for index in (original_count..current).rev() {
                sketches.call(
                    w!("RemoveDataItem"),
                    "removing an incomplete sketch transfer",
                    &mut [Value::from_i32(index)],
                )?;
            }
            Ok(())
        });
        for (_, image) in &loaded {
            let _ = store.call(
                w!("TryRemoveImageData"),
                "removing unused sketch image data",
                &mut [Value::from_dispatch(image.0.clone())],
            );
        }
        return match rollback {
            Ok(()) => Err(error),
            Err(cleanup) => Err(format!(
                "{error}\n{cleanup}\nCheck Additional Sketches before retrying."
            )),
        };
    }
    Ok(())
}
