// Runs inside sketch-app.exe's CLR. No companion process is started.
// These obsolete SDK members are implemented intentionally for legacy hosts.
#pragma warning disable 618
using System;
using System.Collections;
using System.Collections.Generic;
using System.Diagnostics;
using System.Drawing;
using System.Drawing.Imaging;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Security.Principal;
using System.Threading;
using System.Web.Script.Serialization;
using System.Windows.Forms;
using Apex.Integration;
using Apex.Integration.Server;
using Apex.Integration.Client;
[assembly: Bootstrap(typeof(SketchRust.Compat.Bootstrap))]
namespace SketchRust.Compat {
    public sealed class Bootstrap : XmlBootstrapProvider {
        protected override XmlConfiguration GetConfiguration() { return (XmlConfiguration)XmlConfiguration.Read(Session.Config); }
        protected override ProcessStartInfo GetProcessInfo() { throw new NotSupportedException(); }
    }
    public static class Boot {
        static Thread worker;
        static bool resolverInstalled;static Assembly embeddedTransport;
        static void ResolveTransport(){
            lock(typeof(Boot)){if(resolverInstalled)return;resolverInstalled=true;
                AppDomain.CurrentDomain.AssemblyResolve+=delegate(object sender,ResolveEventArgs args){
                    if(new AssemblyName(args.Name).Name!="Apex.Integration.DotNet")return null;
                    lock(typeof(Boot)){if(embeddedTransport!=null)return embeddedTransport;
                        using(var input=typeof(Boot).Assembly.GetManifestResourceStream("SketchRust.HostContract")){
                            if(input==null)throw new FileNotFoundException("The embedded desktop host contract is missing");
                            using(var output=new MemoryStream()){input.CopyTo(output);embeddedTransport=Assembly.Load(output.ToArray());return embeddedTransport;}
                        }
                    }
                };
            }
        }
        public static int ImportCodes(string argsFile){return BootCodes.Import(argsFile);}
        public static int ProbeEmbedded(string ignored){
            using(var input=typeof(Boot).Assembly.GetManifestResourceStream("SketchRust.HostContract")){
                if(input==null)throw new FileNotFoundException("Embedded transport is missing");using(var bytes=new MemoryStream()){input.CopyTo(bytes);var assembly=Assembly.Load(bytes.ToArray());if(assembly.GetName().Name!="Apex.Integration.DotNet"||assembly.GetType("Apex.Integration.Server.SketchServer",true)==null)throw new InvalidDataException("Invalid embedded host transport");}
            }return Process.GetCurrentProcess().Id;
        }
        public static int Probe(string ignored){ResolveTransport();return ProbeTransport();}
        static int ProbeTransport(){GC.KeepAlive(typeof(SketchServer).Assembly);return Process.GetCurrentProcess().Id;}
        public static int Start(string argsFile) {
            ResolveTransport();
            if(worker != null) return 1;
            worker = new Thread(delegate() {
                try {
                    // A managed entry assembly gives the legacy SDK its bootstrap attribute.
                    var domain = AppDomain.CreateDomain("SketchRustApexSession");
                    try {domain.ExecuteAssembly(typeof(Boot).Assembly.Location, new string[]{argsFile});}
                    finally {AppDomain.Unload(domain);}
                } catch(Exception e) { File.WriteAllText(Path.Combine(Path.GetDirectoryName(argsFile),"error.txt"), e.ToString()); }
                finally {File.WriteAllText(Path.Combine(Path.GetDirectoryName(argsFile),"stopped"),"");}
            });
            worker.IsBackground = true; worker.SetApartmentState(ApartmentState.STA); worker.Start(); return 0;
        }
        [STAThread] public static void Main(string[] args) {
            ResolveTransport();
            var boot = Session.Json.Deserialize<Dictionary<string,string>>(File.ReadAllText(args[0]));
            Session.Root = boot["session"]; Session.Config = boot.ContainsKey("config")?boot["config"]:"";
            if(boot.ContainsKey("mode")){LegacyHost.Run(boot);return;}
            RunSdk(boot);
        }
        static void RunSdk(Dictionary<string,string> boot){
            Session.EditorExpected=boot.ContainsKey("gui")&&boot["gui"]=="true";
            // The host SDK queries this per-user marker to detect its sketch
            // provider. Keep a handle for this session without taking ownership.
            using(var marker=new Mutex(false,"Local\\Apex Sketch$"+WindowsIdentity.GetCurrent().Name.Replace("\\","_")))
            using(var server = new Server()) {
                var timer = new System.Windows.Forms.Timer(); timer.Interval = 100;
                timer.Tick += delegate { server.Poll(); }; timer.Start();
                File.WriteAllText(Path.Combine(Session.Root,"server-ready"), Process.GetCurrentProcess().Id.ToString());
                Application.Run(); timer.Dispose();
            }
        }
    }
    public static class Session {
        public static string Root, Config; public static bool EditorExpected;
        public static readonly JavaScriptSerializer Json = new JavaScriptSerializer { MaxJsonLength = 64*1024*1024 };
        public static void AtomicBytes(string path, byte[] bytes) {
            string tmp=path+".new";File.WriteAllBytes(tmp,bytes);if(File.Exists(path))File.Replace(tmp,path,null);else File.Move(tmp,path);
        }
        public static void Atomic(string path, string text) {
            string tmp = path+".new"; File.WriteAllText(tmp,text);
            if(File.Exists(path)) File.Replace(tmp,path,null); else File.Move(tmp,path);
        }
    }
    public sealed class Server : SketchServer {
        Document current; bool ending;
        protected override void OnApplicationClosed() {
            // The embedded SDK enumerates a live list. Pro unregisters its
            // client synchronously during this callback, invalidating it.
            var field=typeof(SketchServer).GetField("clientSinks",BindingFlags.Instance|BindingFlags.NonPublic);
            var clients=((IEnumerable<ISketchClientSink>)field.GetValue(this)).ToArray();
            foreach(var client in clients)client.OnApplicationClosed();
        }
        protected override SketchProxy OnNewSketch(ISketchSink sink, string name) { return Open(sink,name,false); }
        protected override SketchProxy OnOpenSketch(ISketchSink sink,string path,bool base64,int index) {
            if(base64){var parsed=ApexFile.Read(Convert.FromBase64String(path));parsed.ToModel();if(index<0||index>=Math.Max(1,parsed.Pages.Count))throw new ArgumentOutOfRangeException("index");}
            var result=Open(sink,base64?Path.Combine(Session.Root,"Base64.ax7"):path);
            if(base64)result.LoadBytes(Convert.FromBase64String(path));
            result.PageIndex=index; return result;
        }
        protected override void OnCollectGeoReferencePoints(){if(current!=null)current.Request("collect-georef",new{index=current.PageIndex});}
        Document Open(ISketchSink sink,string path,bool existing=true) {
            var next=new Document(sink,path,existing);
            if(current!=null)current.Deactivate();current=next;return current;
        }
        public void Poll() {
            if(ending) return;
            if(current!=null){try{current.Refresh();}catch(IOException){}catch(ArgumentException){}}
            if(File.Exists(Path.Combine(Session.Root,"host-closed"))) {OnApplicationClosed();ending=true;Application.ExitThread();return;}
            if(File.Exists(Path.Combine(Session.Root,"cancel"))) { OnApplicationClosed(); ending=true; Application.ExitThread(); return; }
            string save=Path.Combine(Session.Root,"save-ready");
            if(File.Exists(save)&&current!=null){
                try{current.AcceptReturn();current.AbortReturn();File.Delete(save);Session.Atomic(Path.Combine(Session.Root,"save-receipt.json"),"{\"ok\":true}");}
                catch(Exception error){current.AbortReturn();File.Delete(save);Session.Atomic(Path.Combine(Session.Root,"save-receipt.json"),Session.Json.Serialize(new{ok=false,error=error.Message}));}
                return;
            }
            string ready=Path.Combine(Session.Root,"return-ready");
            if(!File.Exists(ready)||current==null) return;
            try {
                current.AcceptReturn();
                ending=true; OnApplicationClosed();
                Session.Atomic(Path.Combine(Session.Root,"return-receipt.json"),"{\"ok\":true}");
                Application.ExitThread();
            } catch(Exception e) {
                ending=false; if(current!=null)current.AbortReturn();File.Delete(ready);
                Session.Atomic(Path.Combine(Session.Root,"return-receipt.json"),Session.Json.Serialize(new {ok=false,error=e.Message}));
            }
        }
    }
    // Shared collection behavior for the SDK's standard ITagCollection contract.
    public sealed class TagStore {
        readonly List<ITag> items=new List<ITag>();
        public int Count {get{return items.Count;}}
        public ITag At(int index){return items[index];}
        public int IndexOf(string name){if(name==null)throw new ArgumentNullException("name");return items.FindIndex(t=>String.Equals(t.Name,name,StringComparison.Ordinal));}
        public string Get(string name){int i=IndexOf(name);return i<0?null:items[i].Value;}
        public void Set(string name,string value){int i=IndexOf(name);var tag=new ProxyTag(name,value??"");if(i<0)items.Add(tag);else items[i]=tag;}
        public bool Remove(string name){int i=IndexOf(name);if(i<0)return false;items.RemoveAt(i);return true;}
        public void Clear(){items.Clear();}
        public void CopyTo(Array array,int index){((ICollection)items.ToArray()).CopyTo(array,index);}
        public IEnumerator GetEnumerator(){return items.ToArray().GetEnumerator();}
    }
    public sealed class AreaData {
        public int id {get;set;} public string code {get;set;} public string name {get;set;}
        public double baseArea {get;set;} public double netArea {get;set;} public double factor {get;set;} public double perimeter {get;set;}
        public double averageWallHeight {get;set;} public double width {get;set;} public double height {get;set;}
        public string description {get;set;} public Dictionary<string,string> tags {get;set;}
    }
    public sealed class SymbolData {public int id {get;set;}public string name {get;set;}}
    public sealed class GeoData {public double x {get;set;} public double y {get;set;} public string coordinates {get;set;}}
    public sealed class PageData { public string image {get;set;} public AreaData[] areas {get;set;} public float imageDpi {get;set;} public GeoData[] geoReferencePoints {get;set;} public string geoReferenceCoordinateSystem {get;set;} public SymbolData[] symbols {get;set;} }
    public sealed class ReturnData { public PageData[] pages {get;set;} public string revision {get;set;} public void ToNativeUnits(bool metric){if(!metric||pages==null)return;const double f=0.3048;foreach(var page in pages){foreach(var area in page.areas??new AreaData[0]){area.baseArea*=f*f;area.netArea*=f*f;area.perimeter*=f;area.width*=f;area.height*=f;area.averageWallHeight*=f;}foreach(var point in page.geoReferencePoints??new GeoData[0]){point.x*=f;point.y*=f;}}} }
    public sealed class Document : SketchProxy {
        readonly ISketchSink sink; string hostPath; readonly Subject subject;
        ApexFile native,lastEncoded; DateTime snapshotStamp; readonly object requestLock=new object();
        bool refreshing,returning,inactive,isNew; DateTime lastFresh; string revision;
        public void AbortReturn(){returning=false;}
        public void Deactivate(){inactive=true;}
        PageData[] pages = new PageData[]{new PageData {areas=new AreaData[0]}};
        public Document(ISketchSink target,string path,bool existing=true) {
            sink=target;
            hostPath=Path.GetFullPath(String.IsNullOrWhiteSpace(path)?Path.Combine(Session.Root,"Untitled.ax7"):path);
            subject=new Subject(this);isNew=!existing;
            if(existing&&File.Exists(hostPath))LoadBytes(File.ReadAllBytes(hostPath));
            else {
                if(File.Exists(Path.Combine(Session.Root,"editor-ready"))){var blank=ApexFile.Obj("pages",new[]{ApexFile.Obj("lines",new object[0],"polygons",new object[0],"labels",new object[0],"symbols",new object[0])});string file=Path.Combine(Session.Root,"native-new.json");Session.Atomic(file,Session.Json.Serialize(blank));Request("load",new{path=file});}
                Context();
            }
        }
        public void LoadBytes(byte[] bytes) {
            var parsed=ApexFile.Read(bytes);var model=parsed.ToModel();
            string path=Path.Combine(Session.Root,"native-open.json");Session.Atomic(path,Session.Json.Serialize(model));
            if(File.Exists(Path.Combine(Session.Root,"editor-ready")))Request("load",new {path=path});
            native=parsed;subject.Values.Clear();subject.Clear();foreach(var pair in parsed.Subject){if(ApexFile.SubjectKeys.Contains(pair.Key))subject.Values[pair.Key]=pair.Value;else subject[pair.Key]=pair.Value;}
            if(!File.Exists(Path.Combine(Session.Root,"editor-ready"))){pages=parsed.Pages.Select(p=>new PageData{areas=new AreaData[0]}).ToArray();
            if(pages.Length==0)pages=new[]{new PageData{areas=new AreaData[0]}};}
            Context();Refresh();lastFresh=DateTime.MinValue;
        }
        public Dictionary<string,object> Request(string op,object argument=null) {
            if(inactive)throw new ObjectDisposedException("The previous sketch session was replaced");
            if(!File.Exists(Path.Combine(Session.Root,"editor-ready")))return new Dictionary<string,object>();
            lock(requestLock){
                string id=Guid.NewGuid().ToString("N"), response=Path.Combine(Session.Root,"response-"+id+".json");
                Session.Atomic(Path.Combine(Session.Root,"command.json"),Session.Json.Serialize(new{id=id,op=op,argument=argument}));
                int timeout=op=="render"?60000:20000;var watch=Stopwatch.StartNew();while(!File.Exists(response)){
                    if(watch.ElapsedMilliseconds>timeout)throw new TimeoutException("The sketch editor did not complete "+op);
                    if(File.Exists(Path.Combine(Session.Root,"host-closed")))throw new InvalidOperationException("Sketch is closed");Thread.Sleep(10);
                }
                Dictionary<string,object> reply;while(true){try{reply=Session.Json.Deserialize<Dictionary<string,object>>(File.ReadAllText(response));break;}catch(IOException){if(watch.ElapsedMilliseconds>timeout)throw;Thread.Sleep(10);}}try{File.Delete(response);}catch(IOException){}
                if(!reply.ContainsKey("ok")||!Object.Equals(reply["ok"],true))throw new InvalidOperationException(Convert.ToString(reply["error"]));
                return reply;
            }
        }
        public string RenderForm(ImageOptions options,int index){
            var reply=Request("render",new{paper=options.Paper,layout=options.Layout,dpi=options.Dpi,page=index,pageIndex=options.PageIndex});object value;return reply.TryGetValue("image",out value)?Convert.ToString(value):null;
        }
        void Fresh(){if(inactive)throw new ObjectDisposedException("The previous sketch session was replaced");if(refreshing||returning)return;
            if(Session.EditorExpected){var watch=Stopwatch.StartNew();while(!File.Exists(Path.Combine(Session.Root,"editor-ready"))){if(watch.ElapsedMilliseconds>20000)throw new TimeoutException("The native sketch editor did not become ready");Thread.Sleep(10);}}
            if(File.Exists(Path.Combine(Session.Root,"editor-ready"))&&(DateTime.UtcNow-lastFresh).TotalMilliseconds>100){refreshing=true;try{Request("snapshot");Refresh();lastFresh=DateTime.UtcNow;}finally{refreshing=false;}}}
        public void Refresh(){
            if(returning)return;
            string file=Path.Combine(Session.Root,"snapshot.json");if(!File.Exists(file))return;
            DateTime stamp=File.GetLastWriteTimeUtc(file);if(stamp==snapshotStamp)return;
            var value=Session.Json.Deserialize<ReturnData>(File.ReadAllText(file));if(value==null||value.pages==null||value.pages.Length==0)return;
            value.ToNativeUnits(native!=null&&native.Measurement==2);var old=pages;pages=value.pages;lock(pageObjects){foreach(var pair in pageObjects)if(pair.Key<pages.Length)pair.Value.Update(pages[pair.Key]);}snapshotStamp=stamp;
            if(value.revision!=revision){revision=value.revision;bool before=refreshing;refreshing=true;try{
                for(int index=0;index<Math.Max(old.Length,pages.Length);index++){
                    var prior=index<old.Length?old[index].areas:new AreaData[0];var after=index<pages.Length?pages[index].areas:new AreaData[0];
                    foreach(var area in prior)if(!after.Any(a=>a.id==area.id))sink.OnAreaRemoved(new SketchItemEventArgs(index,Array.IndexOf(prior,area),area.id));
                    foreach(var area in after){var previous=prior.FirstOrDefault(a=>a.id==area.id);var args=new SketchItemEventArgs(index,Array.IndexOf(after,area),area.id);
                        if(previous==null)sink.OnAreaAdded(args);else if(previous.netArea!=area.netArea||previous.name!=area.name||previous.code!=area.code||previous.perimeter!=area.perimeter||previous.factor!=area.factor||previous.averageWallHeight!=area.averageWallHeight||Session.Json.Serialize(previous.tags)!=Session.Json.Serialize(area.tags))sink.OnAreaChanged(args);
                    }
                    var oldSymbols=index<old.Length?(old[index].symbols??new SymbolData[0]):new SymbolData[0];var newSymbols=index<pages.Length?(pages[index].symbols??new SymbolData[0]):new SymbolData[0];
                    foreach(var symbol in oldSymbols)if(!newSymbols.Any(s=>s.id==symbol.id))sink.OnSymbolRemoved(new SymbolEventArgs(symbol.id,symbol.name));
                    foreach(var symbol in newSymbols)if(!oldSymbols.Any(s=>s.id==symbol.id))sink.OnSymbolAdded(new SymbolEventArgs(symbol.id,symbol.name));
                    if(index<pages.Length){var beforePoints=index<old.Length?(old[index].geoReferencePoints??new GeoData[0]):new GeoData[0];var afterPoints=pages[index].geoReferencePoints??new GeoData[0];if(beforePoints.Length!=afterPoints.Length||beforePoints.Where((p,i)=>i<afterPoints.Length&&(p.x!=afterPoints[i].x||p.y!=afterPoints[i].y)).Any())sink.OnGeoReferencePointsPlaced();}
                }sink.OnModified();
            }finally{refreshing=before;}}

        }
        public string StatePath { get { return hostPath+".sketch-rust.skx"; } }
        public void Context() {
            lock(this) Session.Atomic(Path.Combine(Session.Root,"context.json"),Session.Json.Serialize(new {
                version=1, hostPath=hostPath, document=native!=null?Path.Combine(Session.Root,"native-open.json"):(!isNew&&File.Exists(StatePath)?StatePath:null),
                existingApex=File.Exists(hostPath), subject=subject.Values, pageIndex=pageIndex, processId=Process.GetCurrentProcess().Id
            }));
        }
        readonly Dictionary<int,Page> pageObjects=new Dictionary<int,Page>();
        public override PageProxy this[int index] {get {
            Fresh();if(index<0||index>=pages.Length) throw new ArgumentOutOfRangeException("index");
            lock(pageObjects){Page page; if(!pageObjects.TryGetValue(index,out page)) pageObjects[index]=page=new Page(pages[index],this,index); return page;}
        }}
        public override int Count {get {Fresh();return pages.Length;}}
        int pageIndex;
        public override int PageIndex {get{return pageIndex;}set{
            if(value<0||value>=pages.Length) throw new ArgumentOutOfRangeException("value");
            if(pageIndex!=value){Request("page",new{index=value});pageIndex=value;Context();sink.OnPageChanged();}
        }}
        public override string Name {get {return hostPath;}}
        public override SubjectProxy Subject {get {return subject;}}
        public override void Close(bool prompt) { if(inactive)return;if(prompt&&sink.OnClosing())return;File.WriteAllText(Path.Combine(Session.Root,"host-closed"),""); sink.OnClosed(); }
        public override void Load(string path,bool base64,int page) {LoadBytes(base64?Convert.FromBase64String(path):File.ReadAllBytes(path));if(!base64)hostPath=Path.GetFullPath(path);PageIndex=page;Context();sink.OnLoaded();}
        public override void Save() {Save(hostPath);}
        public override void Save(string path) {
            byte[] bytes=NativeBytes();string target=Path.GetFullPath(path);Session.AtomicBytes(target,bytes);hostPath=target;
            native=lastEncoded;isNew=false;Session.Atomic(Path.Combine(Session.Root,"native-open.json"),File.ReadAllText(Path.Combine(Session.Root,"model.json")));Context();sink.OnSaved();
        }
        byte[] NativeBytes(bool frozen=false){
            if(!frozen)Fresh();string path=Path.Combine(Session.Root,frozen&&File.Exists(Path.Combine(Session.Root,"return-model.json"))?"return-model.json":"model.json");
            if(!File.Exists(path))throw new InvalidOperationException("The editor has not supplied a document snapshot");
            var model=Session.Json.Deserialize<Dictionary<string,object>>(File.ReadAllText(path));
            var output=ApexFile.FromModel(model,native);
            foreach(var key in output.Subject.Keys.Where(k=>!ApexFile.SubjectKeys.Contains(k)).ToArray())output.Subject.Remove(key);
            foreach(var pair in subject.Values)output.Subject[pair.Key]=pair.Value;
            foreach(ITag tag in subject)output.Subject[tag.Name]=tag.Value;
            byte[] bytes=output.Write();ApexFile.Read(bytes);lastEncoded=output;return bytes;
        }
        public override string SaveAsBase64String() {return Convert.ToBase64String(NativeBytes());}
        public override void ExportPageImages(string path,string options) {
            Fresh();var parsed=ImageOptions.Parse(options,PageIndex,pages.Length);
            for(int i=parsed.First;i<=parsed.Last;i++){
                string target=parsed.Single?path:Path.Combine(Path.GetDirectoryName(path)??"",Path.GetFileNameWithoutExtension(path)+"-"+(i+1)+Path.GetExtension(path));
                this[i].SaveImage(target,options);
            }
        }
        public override string ExportPageAsBase64(string options) {
            Fresh();string forced=(options??"")+" PAGE:CURRENT";var parsed=ImageOptions.Parse(forced,PageIndex,pages.Length);
            string temp=Path.Combine(Session.Root,"image-"+Guid.NewGuid().ToString("N")+(parsed.Form?".emf":".jpg"));
            try{this[PageIndex].SaveImage(temp,forced);return Convert.ToBase64String(File.ReadAllBytes(temp));}finally{if(File.Exists(temp))File.Delete(temp);}
        }
        public override bool ZoomToItem(int id) {Fresh();if(!pages.Any(p=>p.areas.Any(a=>a.id==id)))return false;Request("zoom",new{id=id});return true;}
        public void AcceptReturn() {
            returning=true;
            var value=Session.Json.Deserialize<ReturnData>(File.ReadAllText(Path.Combine(Session.Root,"result.json")));
            if(value==null||value.pages==null||value.pages.Length==0||value.pages.Length>256) throw new InvalidDataException("No sketch pages to return.");
            value.ToNativeUnits(native!=null&&native.Measurement==2);
            foreach(var page in value.pages) {
                string full=Path.GetFullPath(page.image), allowed=Path.GetFullPath(Session.Root)+Path.DirectorySeparatorChar;
                if(!full.StartsWith(allowed,StringComparison.OrdinalIgnoreCase)||!File.Exists(full)) throw new InvalidDataException("Invalid sketch image.");
                if(page.areas==null) throw new InvalidDataException("Missing area data.");
                var ids=new HashSet<int>();
                foreach(var area in page.areas) {
                    if(!ids.Add(area.id)) throw new InvalidDataException("Duplicate area identity.");
                    foreach(double measure in new[]{area.baseArea,area.netArea,area.factor,area.perimeter})
                        if(Double.IsNaN(measure)||Double.IsInfinity(measure)) throw new InvalidDataException("Invalid area measurement.");
                }
                using(var image=Image.FromFile(full)) { if(image.Width<1||image.Height<1) throw new InvalidDataException("Invalid sketch image."); }
            }
            string state=Path.Combine(Session.Root,File.Exists(Path.Combine(Session.Root,"return-document.skx"))?"return-document.skx":"document.skx");
            if(!File.Exists(state)) throw new InvalidDataException("Missing editable sketch.");
            // Generate and validate native bytes before changing either persisted file.
            byte[] nativeBytes=File.Exists(Path.Combine(Session.Root,"model.json"))?NativeBytes(true):null;
            byte[] bytes=File.ReadAllBytes(state); string tmp=StatePath+".new"; File.WriteAllBytes(tmp,bytes);
            if(File.Exists(StatePath)) File.Replace(tmp,StatePath,null); else File.Move(tmp,StatePath);
            if(nativeBytes!=null){Session.AtomicBytes(hostPath,nativeBytes);native=lastEncoded;isNew=false;string frozen=Path.Combine(Session.Root,File.Exists(Path.Combine(Session.Root,"return-model.json"))?"return-model.json":"model.json");Session.Atomic(Path.Combine(Session.Root,"native-open.json"),File.ReadAllText(frozen));Context();}
            pages=value.pages;foreach(var pair in pageObjects)if(pair.Key<pages.Length)pair.Value.Update(pages[pair.Key]);
            if(pageIndex>=pages.Length) pageIndex=0;
            sink.OnSaved();
        }
    }
    public sealed class ImageOptions {
        public int First,Last;public bool Single,Form,Integration,PageIndex;public string Layout="CALCULATIONS",Paper="LEGAL";public float Dpi=300;public EmfType Emf=EmfType.EmfPlusDual;
        public static ImageOptions Parse(string options,int current,int count){
            var parsed=new ImageOptions{First=0,Last=count-1};var tokens=(options??"").Split(new[]{' ','\t','\r','\n'},StringSplitOptions.RemoveEmptyEntries);
            for(int i=0;i<tokens.Length;i++){
                string token=tokens[i];int colon=token.IndexOf(':');string key=(colon<0?token:token.Substring(0,colon)).ToUpperInvariant();string value=colon<0?"":token.Substring(colon+1).ToUpperInvariant();if(colon>=0&&value.Length==0&&i+1<tokens.Length)value=tokens[++i].ToUpperInvariant();
                switch(key){
                    case "PAGEINDEX":parsed.PageIndex=true;break;
                    case "PAGE":case "PAGES":
                        if(colon<0)throw new ArgumentException("Missing page option value");if(key=="PAGE")parsed.Single=true;
                        if(value=="CURRENT"){parsed.First=current;parsed.Last=current;}else if(value=="ALL"){parsed.First=0;parsed.Last=count-1;}else{int page;if(!Int32.TryParse(value,out page)||page<0||page>=count)throw new ArgumentOutOfRangeException("options","Page is outside the sketch");parsed.First=page;parsed.Last=page;}break;
                    case "EMFDPI":float dpi;if(!System.Single.TryParse(value,System.Globalization.NumberStyles.Float,System.Globalization.CultureInfo.InvariantCulture,out dpi)||dpi<50||dpi>2400||System.Single.IsNaN(dpi))throw new ArgumentOutOfRangeException("options","EMF DPI must be 50 through 2400");parsed.Dpi=dpi;break;
                    case "EMFTYPE":switch(value){case "GDI":parsed.Emf=EmfType.EmfOnly;break;case "GDIPLUS":case "GDI+":parsed.Emf=EmfType.EmfPlusOnly;break;case "DUAL":case "BOTH":parsed.Emf=EmfType.EmfPlusDual;break;default:throw new ArgumentException("Unsupported EMFTYPE");}break;
                    case "FORM":parsed.Form=true;switch(value){case "":break;case "INTEGRATION":parsed.Integration=true;break;case "LEGAL":case "LETTER":case "A4":parsed.Paper=value;break;case "STANDARD":case "SKETCHONLY":case "FULLPAGE":case "CALCULATIONS":parsed.Layout=value;break;default:throw new ArgumentException("Unknown form parameter");}break;
                    default:throw new ArgumentException("Unknown image options parameter: "+key);
                }
            }return parsed;
        }
        public static int Page(string options,int current,int count){return Parse(options,current,count).First;}
        public static ImageFormat Format(string path){switch(Path.GetExtension(path).ToLowerInvariant()){case ".png":return ImageFormat.Png;case ".jpg":case ".jpeg":return ImageFormat.Jpeg;case ".bmp":return ImageFormat.Bmp;case ".gif":return ImageFormat.Gif;case ".tif":case ".tiff":return ImageFormat.Tiff;default:throw new NotSupportedException("Unsupported sketch image extension: "+Path.GetExtension(path));}}
    }
    public sealed class Page : PageProxy {
        [System.Runtime.InteropServices.DllImport("gdi32.dll")]
        static extern int GetDeviceCaps(IntPtr hdc,int index);
        PageData data;readonly Document owner;readonly int pageIndex;
        public Page(PageData value,Document document=null,int index=0) { data=value;owner=document;pageIndex=index; }
        public void Update(PageData value){lock(areas){data=value;foreach(var a in value.areas){Area area;if(areas.TryGetValue(a.id,out area))area.Update(a);}}}
        readonly Dictionary<int,Area> areas=new Dictionary<int,Area>();
        public override AreaProxy this[int index] {get {
            if(index<0||index>=data.areas.Length)throw new ArgumentOutOfRangeException("index");
            lock(areas){Area area;int id=data.areas[index].id;if(!areas.TryGetValue(id,out area))areas[id]=area=new Area(data.areas[index],owner);return area;}
        }}
        public override int Count {get {return data.areas.Length;}}
        public override int GeoReferencePointCount {get {return data.geoReferencePoints==null?0:data.geoReferencePoints.Length;}}
        public override string GeoReferenceCoordinateSystem {get{return String.IsNullOrEmpty(data.geoReferenceCoordinateSystem)?"GEOGCS[\"WGS 84\",DATUM[\"WGS_1984\",SPHEROID[\"WGS 84\",6378137,298.257223563]],PRIMEM[\"Greenwich\",0],UNIT[\"degree\",0.0174532925199433]]":data.geoReferenceCoordinateSystem;}set{if(String.IsNullOrWhiteSpace(value))throw new ArgumentNullException("value");if(owner!=null){owner.Request("georef",new{page=pageIndex,operation="system",value=value});owner.Refresh();}data.geoReferenceCoordinateSystem=value;}}
        public override GeoReferencePointProxy GetGeoReferencePoint(int index) {
            if(index<0||index>=GeoReferencePointCount)throw new ArgumentOutOfRangeException("index");
            return new GeoPoint(()=>data.geoReferencePoints[index],value=>{if(owner!=null){owner.Request("georef",new{page=pageIndex,index=index,operation="coordinates",value=value});owner.Refresh();}else data.geoReferencePoints[index].coordinates=value;});
        }
        public override SketchImage ToSketchImage(){using(var image=Image.FromFile(data.image))return new SketchImage(image);}
        public override void SaveImage(string path,string options) {
            var parsed=ImageOptions.Parse(options,pageIndex,owner==null?Math.Max(pageIndex+1,1):owner.Count);
            string source=data.image;float dpi=data.imageDpi;
            if(parsed.Form&&!parsed.Integration&&owner!=null){string rendered=owner.RenderForm(parsed,pageIndex);if(rendered!=null){source=rendered;dpi=parsed.Dpi;}}
            using(var input=Image.FromFile(source))using(var image=new Bitmap(input)) {
                if(dpi>0)image.SetResolution(dpi,dpi);
                if(parsed.Form||Path.GetExtension(path).Equals(".emf",StringComparison.OrdinalIgnoreCase)) {
                    using(var bitmap=new Bitmap(1,1)) using(var context=Graphics.FromImage(bitmap)) {
                        IntPtr hdc=context.GetHdc();
                        try {
                            var frame=new RectangleF(0,0,image.Width*25.4f/image.HorizontalResolution,image.Height*25.4f/image.VerticalResolution);
                            using(var meta=new Metafile(path,hdc,frame,MetafileFrameUnit.Millimeter,parsed.Emf)) {
                              int mmWidth=GetDeviceCaps(hdc,4),mmHeight=GetDeviceCaps(hdc,6);
                              if(mmWidth<=0||mmHeight<=0)throw new InvalidOperationException("Invalid metafile reference device");
                              int pixelsX=GetDeviceCaps(hdc,118),pixelsY=GetDeviceCaps(hdc,117);
                              if(pixelsX<=0)pixelsX=GetDeviceCaps(hdc,8);
                              if(pixelsY<=0)pixelsY=GetDeviceCaps(hdc,10);
                              var bounds=new RectangleF(0,0,frame.Width*pixelsX/mmWidth,frame.Height*pixelsY/mmHeight);
                              using(var canvas=Graphics.FromImage(meta)) {
                                // Metafile device DPI can differ from the
                                // recording Graphics DPI (especially over RDP).
                                // Draw in the frame's pixel coordinates so the
                                // report fills it without right/bottom padding.
                                canvas.PageUnit=GraphicsUnit.Pixel;
                                canvas.Clear(Color.White);
                                // Account for a DPI-virtualized recording
                                // Graphics as well as DPI-aware sketch windows.
                                bounds.Width*=GetDeviceCaps(hdc,88)/canvas.DpiX;
                                bounds.Height*=GetDeviceCaps(hdc,90)/canvas.DpiY;
                                canvas.DrawImage(image,bounds,new RectangleF(0,0,image.Width,image.Height),GraphicsUnit.Pixel);
                              }
                            }
                        } finally {context.ReleaseHdc(hdc);}
                    }
                } else image.Save(path,ImageOptions.Format(path));
            }
        }
        public override void Save(ImageFormat format,string path) {using(var img=Image.FromFile(data.image)) img.Save(path,format);}
        public override void Save(SketchImageFormat format,string path) {
            switch(format){
                case SketchImageFormat.Bmp:Save(ImageFormat.Bmp,path);break;
                case SketchImageFormat.Jpeg:Save(ImageFormat.Jpeg,path);break;
                case SketchImageFormat.Gif:Save(ImageFormat.Gif,path);break;
                case SketchImageFormat.Tiff:Save(ImageFormat.Tiff,path);break;
                case SketchImageFormat.Png:Save(ImageFormat.Png,path);break;
                default:throw new ArgumentOutOfRangeException("format");
            }
        }
    }
    public sealed class GeoPoint : GeoReferencePointProxy {
        readonly Func<GeoData> get;readonly Action<string> set;
        public GeoPoint(Func<GeoData> read,Action<string> write){get=read;set=write;}
        public override double SketchX {get{return get().x;}}
        public override double SketchY {get{return get().y;}}
        public override string GeoCoordinates {get{return get().coordinates??"";}set{if(value==null)throw new ArgumentNullException("value");set(value);}}
    }
    public sealed class Area : AreaProxy {
        AreaData data;readonly Document owner;public Area(AreaData value,Document document=null){owner=document;Update(value);}
        public void Update(AreaData value){data=value;tags.Clear();if(value.tags!=null)foreach(var pair in value.tags)tags.Set(pair.Key,pair.Value);}
        public override string Code {get{return data.code;}}
        public override string Description {get{return data.description??data.name;}}
        public override int UniqueId {get{return data.id;}}
        public override double Multiplier {get{return data.factor;}}
        public override string Name {get{return data.name;}}
        public override double BaseArea {get{return data.baseArea;}}
        public override double NetArea {get{return data.netArea;}}
        public override double Perimeter {get{return data.perimeter;}}
        public override int Sign {get{return data.factor<0||data.netArea<0?-1:1;}}
        public override double AverageWallHeight {get{return data.averageWallHeight;}}
        public override double WallArea {get{return data.averageWallHeight*data.perimeter;}}
        public override void Delete(){if(owner==null)throw new InvalidOperationException("Area is detached");owner.Request("delete",new{id=data.id});owner.Refresh();}
        public override void GetBreakdown(out char[] shapes,out double[,] metrics){shapes=new[]{Math.Abs(data.width*data.height-data.baseArea)<1e-7?'R':'P'};metrics=new double[,]{{data.baseArea,data.width,data.height}};}
        readonly TagStore tags=new TagStore();
        public override int Count {get{return tags.Count;}}
        public override ITag this[int index] {get{return tags.At(index);}}
        public override string this[string name] {get{return tags.Get(name);}set{if(owner!=null){owner.Request("tag",new{id=data.id,name=name,value=value,operation="set"});owner.Refresh();}tags.Set(name,value);}}
        public override object SyncRoot {get{return tags;}}
        public override bool IsSynchronized {get{return false;}}
        public override void CopyTo(Array array,int index){tags.CopyTo(array,index);}
        public override bool Remove(string name){bool present=tags.IndexOf(name)>=0;if(present&&owner!=null){owner.Request("tag",new{id=data.id,name=name,operation="remove"});owner.Refresh();}tags.Remove(name);return present;}
        public override bool Contains(string name){return tags.IndexOf(name)>=0;}
        public override int IndexOf(string name){return tags.IndexOf(name);}
        public override void Clear(){if(owner!=null){owner.Request("tag",new{id=data.id,operation="clear"});owner.Refresh();}tags.Clear();}
        public override IEnumerator GetEnumerator(){return tags.GetEnumerator();}
    }
    public sealed class Subject : SubjectProxy {
        readonly Document owner; public readonly Dictionary<string,string> Values=new Dictionary<string,string>();
        public Subject(Document value){owner=value;}
        string Get(string key){string v;return Values.TryGetValue(key,out v)?v:"";}
        void Put(string key,string v){Values[key]=v??"";if(File.Exists(Path.Combine(Session.Root,"editor-ready")))owner.Request("subject",new{key=key,value=v??""});owner.Context();}
        public override string Appraiser {get{return Get("Appraiser");}set{Put("Appraiser",value);}}
        public override string AppraiserAddress {get{return Get("AppraiserAddress");}set{Put("AppraiserAddress",value);}}
        public override string Borrower {get{return Get("Borrower");}set{Put("Borrower",value);}}
        public override string CaseNumber {get{return Get("CaseNumber");}set{Put("CaseNumber",value);}}
        public override string City {get{return Get("City");}set{Put("City",value);}}
        public override string County {get{return Get("County");}set{Put("County",value);}}
        public override string FileNumber {get{return Get("FileNumber");}set{Put("FileNumber",value);}}
        public override string LenderClient {get{return Get("LenderClient");}set{Put("LenderClient",value);}}
        public override string LenderClientAddress {get{return Get("LenderClientAddress");}set{Put("LenderClientAddress",value);}}
        public override string PropertyAddress {get{return Get("PropertyAddress");}set{Put("PropertyAddress",value);}}
        public override string State {get{return Get("State");}set{Put("State",value);}}
        public override string ZipCode {get{return Get("ZipCode");}set{Put("ZipCode",value);}}
        readonly TagStore tags=new TagStore();
        public override int Count {get{return tags.Count;}}
        public override ITag this[int index] {get{return tags.At(index);}}
        public override string this[string name] {get{return tags.Get(name);}set{tags.Set(name,value);}}
        public override object SyncRoot {get{return tags;}}
        public override bool IsSynchronized {get{return false;}}
        public override void CopyTo(Array array,int index){tags.CopyTo(array,index);}
        public override bool Remove(string name){return tags.Remove(name);}
        public override bool Contains(string name){return tags.IndexOf(name)>=0;}
        public override int IndexOf(string name){return tags.IndexOf(name);}
        public override void Clear(){tags.Clear();}
        public override IEnumerator GetEnumerator(){return tags.GetEnumerator();}
    }
}
