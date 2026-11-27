//! A small reader and writer for the .NET Remoting Binary Format (MS-NRBF),
//! the stream `BinaryFormatter` produces — only the records that RapidSketch
//! sketch files use.
//!
//! Reading builds a plain table of records. No .NET type is ever created and no
//! code runs, so (unlike `BinaryFormatter` itself) opening a hostile file can
//! only fail with an error.

use std::collections::{HashMap, VecDeque};
use std::rc::Rc;

/// Largest stream accepted.
pub const MAX_STREAM: usize = 64 * 1024 * 1024;
/// Most objects, strings and arrays one stream may declare.
const MAX_ITEMS: usize = 2_000_000;
/// Most array slots one stream may declare, all arrays together.
const MAX_SLOTS: usize = 4_000_000;
/// Deepest nesting of inline records (real files nest two or three deep).
const MAX_DEPTH: usize = 64;
/// Most members one class may declare.
const MAX_MEMBERS: usize = 512;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NrbfError {
    Truncated,
    TooLarge,
    Invalid(&'static str),
}

impl std::fmt::Display for NrbfError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NrbfError::Truncated => write!(f, "the file ends too soon"),
            NrbfError::TooLarge => write!(f, "the file is too large"),
            NrbfError::Invalid(why) => write!(f, "{why}"),
        }
    }
}

impl std::error::Error for NrbfError {}

/// One member or array slot.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    /// An object, string or array, by its record id.
    Ref(i32),
}

#[derive(Debug)]
pub struct ClassInfo {
    pub name: String,
    pub members: Vec<String>,
    /// `None` for the records that carry no type information.
    types: Vec<MemberType>,
}

#[derive(Debug, Clone, Copy)]
enum MemberType {
    Primitive(u8),
    Record,
}

#[derive(Debug)]
pub enum Item {
    Object {
        class: Rc<ClassInfo>,
        values: Vec<Value>,
    },
    Str(String),
    Array(Vec<Value>),
    Bytes(usize),
}

/// Every record of a stream, by id.
#[derive(Debug)]
pub struct Graph {
    root: i32,
    items: HashMap<i32, Item>,
}

/// An object in a [`Graph`].
#[derive(Clone, Copy)]
pub struct Node<'a> {
    graph: &'a Graph,
    pub id: i32,
    class: &'a ClassInfo,
    values: &'a [Value],
}

impl Graph {
    pub fn root(&self) -> Option<Node<'_>> {
        self.node(Value::Ref(self.root))
    }

    pub fn node(&self, value: Value) -> Option<Node<'_>> {
        let Value::Ref(id) = value else { return None };
        match self.items.get(&id)? {
            Item::Object { class, values } => Some(Node {
                graph: self,
                id,
                class,
                values,
            }),
            _ => None,
        }
    }
}

impl<'a> Node<'a> {
    pub fn class(&self) -> &'a str {
        &self.class.name
    }

    /// The class name without its namespace.
    pub fn short_class(&self) -> &'a str {
        let name = self.class.name.split('`').next().unwrap_or("");
        name.rsplit('.').next().unwrap_or(name)
    }

    pub fn get(&self, member: &str) -> Value {
        self.class
            .members
            .iter()
            .position(|m| m == member)
            .and_then(|i| self.values.get(i).copied())
            .unwrap_or(Value::Null)
    }

    pub fn child(&self, member: &str) -> Option<Node<'a>> {
        self.graph.node(self.get(member))
    }

    pub fn string(&self, member: &str) -> &'a str {
        match self.get(member) {
            Value::Ref(id) => match self.graph.items.get(&id) {
                Some(Item::Str(s)) => s,
                _ => "",
            },
            _ => "",
        }
    }

    pub fn number(&self, member: &str) -> Option<f64> {
        match self.get(member) {
            Value::Float(v) => Some(v),
            Value::Int(v) => Some(v as f64),
            _ => None,
        }
    }

    pub fn int(&self, member: &str) -> Option<i64> {
        match self.get(member) {
            Value::Int(v) => Some(v),
            _ => None,
        }
    }

    pub fn flag(&self, member: &str) -> Option<bool> {
        match self.get(member) {
            Value::Bool(v) => Some(v),
            _ => None,
        }
    }

    /// The elements of a .NET list: an `ArrayList` itself, or a class that
    /// wraps one in a `List` member.
    pub fn list(&self) -> Vec<Value> {
        let array_list = match self.child("List") {
            Some(inner) => inner,
            None => *self,
        };
        let size = array_list.int("_size").unwrap_or(0).max(0) as usize;
        let Value::Ref(id) = array_list.get("_items") else {
            return Vec::new();
        };
        match self.graph.items.get(&id) {
            Some(Item::Array(items)) => items.iter().take(size).copied().collect(),
            _ => Vec::new(),
        }
    }

    /// The objects in the list held by `member`.
    pub fn children(&self, member: &str) -> Vec<Node<'a>> {
        let Some(list) = self.child(member) else {
            return Vec::new();
        };
        list.list()
            .into_iter()
            .filter_map(|v| self.graph.node(v))
            .collect()
    }

    /// The strings in the list held by `member`.
    pub fn strings(&self, member: &str) -> Vec<&'a str> {
        let Some(list) = self.child(member) else {
            return Vec::new();
        };
        list.list()
            .into_iter()
            .filter_map(|v| match v {
                Value::Ref(id) => match self.graph.items.get(&id) {
                    Some(Item::Str(s)) => Some(s.as_str()),
                    _ => None,
                },
                _ => None,
            })
            .collect()
    }
}

struct Reader<'a> {
    data: &'a [u8],
    at: usize,
    items: HashMap<i32, Item>,
    classes: HashMap<i32, Rc<ClassInfo>>,
    slots: usize,
}

/// What one record turned out to be.
enum Record {
    Value(Value),
    Nulls(usize),
    Library,
    End,
}

impl<'a> Reader<'a> {
    fn bytes(&mut self, n: usize) -> Result<&'a [u8], NrbfError> {
        let end = self.at.checked_add(n).ok_or(NrbfError::Truncated)?;
        let out = self.data.get(self.at..end).ok_or(NrbfError::Truncated)?;
        self.at = end;
        Ok(out)
    }

    fn array<const N: usize>(&mut self) -> Result<[u8; N], NrbfError> {
        let mut out = [0; N];
        out.copy_from_slice(self.bytes(N)?);
        Ok(out)
    }

    fn u8(&mut self) -> Result<u8, NrbfError> {
        Ok(self.bytes(1)?[0])
    }

    fn i32(&mut self) -> Result<i32, NrbfError> {
        Ok(i32::from_le_bytes(self.array()?))
    }

    /// A non-negative count no larger than `limit`.
    fn count(&mut self, limit: usize) -> Result<usize, NrbfError> {
        let n = self.i32()?;
        usize::try_from(n)
            .ok()
            .filter(|n| *n <= limit)
            .ok_or(NrbfError::Invalid("a count is out of range"))
    }

    fn string(&mut self) -> Result<String, NrbfError> {
        let mut length = 0usize;
        for shift in 0..5 {
            let byte = self.u8()?;
            length |= usize::from(byte & 0x7f) << (7 * shift);
            if byte & 0x80 == 0 {
                let raw = self.bytes(length)?;
                return Ok(String::from_utf8_lossy(raw).into_owned());
            }
        }
        Err(NrbfError::Invalid("a string length is malformed"))
    }

    fn primitive(&mut self, kind: u8) -> Result<Value, NrbfError> {
        Ok(match kind {
            1 => Value::Bool(self.u8()? != 0),
            2 => Value::Int(i64::from(self.u8()?)),
            10 => Value::Int(i64::from(self.u8()? as i8)),
            6 => Value::Float(f64::from_le_bytes(self.array()?)),
            11 => Value::Float(f64::from(f32::from_le_bytes(self.array()?))),
            7 => Value::Int(i64::from(i16::from_le_bytes(self.array()?))),
            8 => Value::Int(i64::from(self.i32()?)),
            9 | 12 | 13 | 16 => Value::Int(i64::from_le_bytes(self.array()?)),
            14 => Value::Int(i64::from(u16::from_le_bytes(self.array()?))),
            15 => Value::Int(i64::from(u32::from_le_bytes(self.array()?))),
            // Char: one UTF-8 character.
            3 => {
                let first = self.u8()?;
                let extra = match first {
                    0x00..=0x7f => 0,
                    0xc0..=0xdf => 1,
                    0xe0..=0xef => 2,
                    _ => 3,
                };
                self.bytes(extra)?;
                Value::Null
            }
            // Decimal and String travel as text; neither is used here.
            5 | 18 => {
                self.string()?;
                Value::Null
            }
            17 => Value::Null,
            _ => return Err(NrbfError::Invalid("an unknown primitive type")),
        })
    }

    fn insert(&mut self, id: i32, item: Item) -> Result<(), NrbfError> {
        if self.items.len() >= MAX_ITEMS {
            return Err(NrbfError::TooLarge);
        }
        self.items.insert(id, item);
        Ok(())
    }

    /// The next member or element, skipping library records.
    fn value(&mut self, depth: usize) -> Result<Record, NrbfError> {
        loop {
            match self.record(depth)? {
                Record::Library => continue,
                Record::End => return Err(NrbfError::Truncated),
                other => return Ok(other),
            }
        }
    }

    fn elements(&mut self, length: usize, depth: usize) -> Result<Vec<Value>, NrbfError> {
        self.slots = self.slots.saturating_add(length);
        if self.slots > MAX_SLOTS {
            return Err(NrbfError::TooLarge);
        }
        let mut out = Vec::new();
        while out.len() < length {
            match self.value(depth)? {
                Record::Value(v) => out.push(v),
                Record::Nulls(n) if n <= length - out.len() => {
                    out.resize(out.len() + n, Value::Null)
                }
                _ => return Err(NrbfError::Invalid("an array overruns its length")),
            }
        }
        Ok(out)
    }

    /// `BinaryTypeEnum` additional info; keeps only what reading needs.
    fn type_info(&mut self, binary_type: u8) -> Result<MemberType, NrbfError> {
        Ok(match binary_type {
            0 => MemberType::Primitive(self.u8()?),
            1 | 2 | 5 | 6 => MemberType::Record,
            3 => {
                self.string()?;
                MemberType::Record
            }
            4 => {
                self.string()?;
                self.i32()?;
                MemberType::Record
            }
            7 => {
                self.u8()?;
                MemberType::Record
            }
            _ => return Err(NrbfError::Invalid("an unknown member type")),
        })
    }

    fn members(
        &mut self,
        id: i32,
        class: Rc<ClassInfo>,
        depth: usize,
    ) -> Result<Record, NrbfError> {
        let mut values = Vec::with_capacity(class.types.len());
        for kind in &class.types {
            values.push(match *kind {
                MemberType::Primitive(p) => self.primitive(p)?,
                MemberType::Record => match self.value(depth)? {
                    Record::Value(v) => v,
                    Record::Nulls(1) => Value::Null,
                    _ => return Err(NrbfError::Invalid("a member holds a run of nulls")),
                },
            });
        }
        self.insert(id, Item::Object { class, values })?;
        Ok(Record::Value(Value::Ref(id)))
    }

    fn record(&mut self, depth: usize) -> Result<Record, NrbfError> {
        if depth > MAX_DEPTH {
            return Err(NrbfError::Invalid("records are nested too deeply"));
        }
        let depth = depth + 1;
        Ok(match self.u8()? {
            // ClassWithId
            1 => {
                let id = self.i32()?;
                let metadata = self.i32()?;
                let class = self
                    .classes
                    .get(&metadata)
                    .cloned()
                    .ok_or(NrbfError::Invalid("a record names an unknown class"))?;
                self.members(id, class, depth)?
            }
            // (System)ClassWithMembersAndTypes
            tag @ (4 | 5) => {
                let id = self.i32()?;
                let name = self.string()?;
                let count = self.count(MAX_MEMBERS)?;
                let mut members = Vec::with_capacity(count);
                for _ in 0..count {
                    members.push(self.string()?);
                }
                let binary_types = self.bytes(count)?;
                let mut types = Vec::with_capacity(count);
                for &t in binary_types {
                    types.push(self.type_info(t)?);
                }
                if tag == 5 {
                    self.i32()?;
                }
                let class = Rc::new(ClassInfo {
                    name,
                    members,
                    types,
                });
                self.classes.insert(id, class.clone());
                self.members(id, class, depth)?
            }
            // BinaryObjectString
            6 => {
                let id = self.i32()?;
                let text = self.string()?;
                self.insert(id, Item::Str(text))?;
                Record::Value(Value::Ref(id))
            }
            // BinaryArray
            7 => {
                let id = self.i32()?;
                let shape = self.u8()?;
                let rank = self.count(32)?;
                let mut length = 1usize;
                for _ in 0..rank {
                    length = length.saturating_mul(self.count(MAX_SLOTS)?);
                }
                if shape >= 3 {
                    for _ in 0..rank {
                        self.i32()?;
                    }
                }
                let binary_type = self.u8()?;
                let kind = self.type_info(binary_type)?;
                let items = match kind {
                    MemberType::Primitive(p) => {
                        self.slots = self.slots.saturating_add(length);
                        if self.slots > MAX_SLOTS {
                            return Err(NrbfError::TooLarge);
                        }
                        let mut out = Vec::new();
                        for _ in 0..length {
                            out.push(self.primitive(p)?);
                        }
                        out
                    }
                    MemberType::Record => self.elements(length, depth)?,
                };
                self.insert(id, Item::Array(items))?;
                Record::Value(Value::Ref(id))
            }
            // MemberPrimitiveTyped
            8 => {
                let kind = self.u8()?;
                Record::Value(self.primitive(kind)?)
            }
            9 => Record::Value(Value::Ref(self.i32()?)),
            10 => Record::Value(Value::Null),
            11 => Record::End,
            // BinaryLibrary
            12 => {
                self.i32()?;
                self.string()?;
                Record::Library
            }
            13 => Record::Nulls(usize::from(self.u8()?)),
            14 => Record::Nulls(self.count(MAX_SLOTS)?),
            // ArraySinglePrimitive; a byte array (an embedded picture) is
            // skipped, not kept.
            15 => {
                let id = self.i32()?;
                let length = self.count(MAX_STREAM)?;
                let kind = self.u8()?;
                if kind == 2 {
                    self.bytes(length)?;
                    self.insert(id, Item::Bytes(length))?;
                } else {
                    self.slots = self.slots.saturating_add(length);
                    if self.slots > MAX_SLOTS {
                        return Err(NrbfError::TooLarge);
                    }
                    let mut out = Vec::new();
                    for _ in 0..length {
                        out.push(self.primitive(kind)?);
                    }
                    self.insert(id, Item::Array(out))?;
                }
                Record::Value(Value::Ref(id))
            }
            // ArraySingleObject / ArraySingleString
            16 | 17 => {
                let id = self.i32()?;
                let length = self.count(MAX_SLOTS)?;
                let items = self.elements(length, depth)?;
                self.insert(id, Item::Array(items))?;
                Record::Value(Value::Ref(id))
            }
            _ => return Err(NrbfError::Invalid("an unsupported record type")),
        })
    }
}

/// The 17-byte `SerializationHeaderRecord` starts with a zero tag and ends
/// with version 1.0.
pub fn has_header(bytes: &[u8]) -> bool {
    bytes.len() > 17 && bytes[0] == 0 && bytes[9..17] == [1, 0, 0, 0, 0, 0, 0, 0]
}

/// Parse a whole stream.
pub fn parse(bytes: &[u8]) -> Result<Graph, NrbfError> {
    if bytes.len() > MAX_STREAM {
        return Err(NrbfError::TooLarge);
    }
    if !has_header(bytes) {
        return Err(NrbfError::Invalid("not a .NET binary stream"));
    }
    let mut reader = Reader {
        data: bytes,
        at: 1,
        items: HashMap::new(),
        classes: HashMap::new(),
        slots: 0,
    };
    let root = reader.i32()?;
    reader.at = 17;
    loop {
        if let Record::End = reader.record(0)? {
            break;
        }
    }
    Ok(Graph {
        root,
        items: reader.items,
    })
}

// ---------------------------------------------------------------------------
// Writing
// ---------------------------------------------------------------------------

/// A member's declared type, as it appears in a class record.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Bool,
    I16,
    I32,
    I64,
    F32,
    F64,
    Str,
    Object,
    ObjectArray,
    /// A class from `mscorlib`.
    System(&'static str),
    /// A class from one of the stream's libraries, by index.
    Class(&'static str, usize),
}

/// A class as it is declared in the stream.
#[derive(Debug, PartialEq, Eq)]
pub struct Schema {
    pub name: &'static str,
    /// Index into the library list; `None` for `mscorlib` classes.
    pub library: Option<usize>,
    pub members: &'static [(&'static str, Kind)],
}

/// A member value to write.
#[derive(Debug, Clone)]
pub enum Field {
    Null,
    Bool(bool),
    I16(i16),
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
    Str(String),
    /// Another heap entry, written once and referenced everywhere.
    Ref(usize),
    /// A struct or enum, written in place.
    Inline(&'static Schema, Vec<Field>),
}

#[derive(Debug)]
enum Entry {
    Pending,
    Object(&'static Schema, Vec<Field>),
    /// `ArraySingleObject`: the slots, then nulls up to the capacity.
    Array(Vec<Field>, usize),
    /// An empty array of the named class.
    EmptyArray(&'static str, usize),
}

/// The objects of a stream being built. The first one reserved is the root.
#[derive(Debug, Default)]
pub struct Heap {
    entries: Vec<Entry>,
}

impl Heap {
    /// Claim an id now and fill it later, so objects can point at each other.
    pub fn reserve(&mut self) -> usize {
        self.entries.push(Entry::Pending);
        self.entries.len() - 1
    }

    pub fn set(&mut self, id: usize, schema: &'static Schema, fields: Vec<Field>) {
        debug_assert_eq!(schema.members.len(), fields.len(), "{}", schema.name);
        self.entries[id] = Entry::Object(schema, fields);
    }

    pub fn add(&mut self, schema: &'static Schema, fields: Vec<Field>) -> usize {
        let id = self.reserve();
        self.set(id, schema, fields);
        id
    }

    pub fn array(&mut self, items: Vec<Field>, capacity: usize) -> usize {
        let id = self.reserve();
        let capacity = capacity.max(items.len());
        self.entries[id] = Entry::Array(items, capacity);
        id
    }

    pub fn empty_array(&mut self, class: &'static str, library: usize) -> usize {
        let id = self.reserve();
        self.entries[id] = Entry::EmptyArray(class, library);
        id
    }
}

struct Writer<'a> {
    out: Vec<u8>,
    heap: &'a Heap,
    /// Stream id of each heap entry already referenced (0 = not yet).
    ids: Vec<i32>,
    queue: VecDeque<usize>,
    next_id: i32,
    next_inline: i32,
    /// First object written for each class, which later ones point back to.
    classes: HashMap<&'static str, i32>,
    strings: HashMap<String, i32>,
    /// Stream id of each library, by index.
    libraries: Vec<i32>,
}

impl Writer<'_> {
    fn i32(&mut self, v: i32) {
        self.out.extend(v.to_le_bytes());
    }

    fn string(&mut self, text: &str) {
        let mut length = text.len();
        loop {
            let byte = (length & 0x7f) as u8;
            length >>= 7;
            if length == 0 {
                self.out.push(byte);
                break;
            }
            self.out.push(byte | 0x80);
        }
        self.out.extend(text.as_bytes());
    }

    fn fresh_id(&mut self) -> i32 {
        self.next_id += 1;
        self.next_id
    }

    fn id_of(&mut self, entry: usize) -> i32 {
        if self.ids[entry] == 0 {
            self.ids[entry] = self.fresh_id();
            self.queue.push_back(entry);
        }
        self.ids[entry]
    }

    fn nulls(&mut self, count: usize) {
        match count {
            0 => {}
            1 => self.out.push(10),
            2..=255 => self.out.extend([13, count as u8]),
            _ => {
                self.out.push(14);
                self.i32(count as i32);
            }
        }
    }

    fn field(&mut self, field: &Field) {
        match field {
            Field::Null => self.out.push(10),
            Field::Bool(v) => self.out.push(u8::from(*v)),
            Field::I16(v) => self.out.extend(v.to_le_bytes()),
            Field::I32(v) => self.i32(*v),
            Field::I64(v) => self.out.extend(v.to_le_bytes()),
            Field::F32(v) => self.out.extend(v.to_le_bytes()),
            Field::F64(v) => self.out.extend(v.to_le_bytes()),
            Field::Str(text) => match self.strings.get(text).copied() {
                Some(id) => {
                    self.out.push(9);
                    self.i32(id);
                }
                None => {
                    let id = self.fresh_id();
                    self.strings.insert(text.clone(), id);
                    self.out.push(6);
                    self.i32(id);
                    self.string(text);
                }
            },
            Field::Ref(entry) => {
                let id = self.id_of(*entry);
                self.out.push(9);
                self.i32(id);
            }
            Field::Inline(schema, fields) => {
                self.next_inline -= 1;
                let id = self.next_inline;
                self.object(id, schema, fields);
            }
        }
    }

    fn object(&mut self, id: i32, schema: &'static Schema, fields: &[Field]) {
        if let Some(&metadata) = self.classes.get(schema.name) {
            self.out.push(1);
            self.i32(id);
            self.i32(metadata);
        } else {
            self.classes.insert(schema.name, id);
            self.out.push(if schema.library.is_some() { 5 } else { 4 });
            self.i32(id);
            self.string(schema.name);
            self.i32(schema.members.len() as i32);
            for (name, _) in schema.members {
                self.string(name);
            }
            for (_, kind) in schema.members {
                self.out.push(match kind {
                    Kind::Bool | Kind::I16 | Kind::I32 | Kind::I64 | Kind::F32 | Kind::F64 => 0,
                    Kind::Str => 1,
                    Kind::Object => 2,
                    Kind::System(_) => 3,
                    Kind::Class(..) => 4,
                    Kind::ObjectArray => 5,
                });
            }
            for (_, kind) in schema.members {
                match kind {
                    Kind::Bool => self.out.push(1),
                    Kind::F64 => self.out.push(6),
                    Kind::I16 => self.out.push(7),
                    Kind::I32 => self.out.push(8),
                    Kind::I64 => self.out.push(9),
                    Kind::F32 => self.out.push(11),
                    Kind::System(name) => self.string(name),
                    Kind::Class(name, library) => {
                        self.string(name);
                        let library = self.libraries[*library];
                        self.i32(library);
                    }
                    Kind::Str | Kind::Object | Kind::ObjectArray => {}
                }
            }
            if let Some(library) = schema.library {
                let library = self.libraries[library];
                self.i32(library);
            }
        }
        for field in fields {
            self.field(field);
        }
    }

    fn entry(&mut self, entry: usize) {
        let id = self.ids[entry];
        let heap = self.heap;
        match &heap.entries[entry] {
            Entry::Pending => self.out.push(10),
            Entry::Object(schema, fields) => self.object(id, schema, fields),
            Entry::Array(items, capacity) => {
                self.out.push(16);
                self.i32(id);
                self.i32(*capacity as i32);
                for item in items {
                    self.field(item);
                }
                self.nulls(capacity - items.len());
            }
            Entry::EmptyArray(class, library) => {
                self.out.push(7);
                self.i32(id);
                self.out.push(0); // single-dimensional
                self.i32(1);
                self.i32(0);
                self.out.push(4);
                self.string(class);
                let library = self.libraries[*library];
                self.i32(library);
            }
        }
    }
}

/// Write `heap` (entry 0 is the root), declaring the named libraries.
pub fn write(heap: &Heap, libraries: &[&str]) -> Vec<u8> {
    let mut writer = Writer {
        out: Vec::new(),
        heap,
        ids: vec![0; heap.entries.len()],
        queue: VecDeque::new(),
        next_id: 0,
        next_inline: 0,
        classes: HashMap::new(),
        strings: HashMap::new(),
        libraries: Vec::new(),
    };
    writer.out.push(0);
    writer.i32(1); // root id
    writer.i32(-1);
    writer.i32(1);
    writer.i32(0);
    if heap.entries.is_empty() {
        writer.out.push(11);
        return writer.out;
    }
    writer.id_of(0); // the root takes id 1
    for name in libraries {
        let id = writer.fresh_id();
        writer.libraries.push(id);
        writer.out.push(12);
        writer.i32(id);
        writer.string(name);
    }
    while let Some(entry) = writer.queue.pop_front() {
        writer.entry(entry);
    }
    writer.out.push(11);
    writer.out
}

#[cfg(test)]
mod tests {
    use super::*;

    static NODE: Schema = Schema {
        name: "Test.Node",
        library: Some(0),
        members: &[
            ("name", Kind::Str),
            ("count", Kind::I32),
            ("next", Kind::Class("Test.Node", 0)),
            ("items", Kind::ObjectArray),
        ],
    };

    #[test]
    fn a_cycle_round_trips() {
        let mut heap = Heap::default();
        let a = heap.reserve();
        let items = heap.array(vec![Field::Str("x".into())], 300);
        let b = heap.add(
            &NODE,
            vec![
                Field::Str("b".into()),
                Field::I32(2),
                Field::Ref(a),
                Field::Null,
            ],
        );
        heap.set(
            a,
            &NODE,
            vec![
                Field::Str("a".repeat(200)),
                Field::I32(1),
                Field::Ref(b),
                Field::Ref(items),
            ],
        );
        let graph = parse(&write(&heap, &["Test, Version=1.0.0.0"])).unwrap();
        let root = graph.root().unwrap();
        assert_eq!(root.string("name").len(), 200);
        let next = root.child("next").unwrap();
        assert_eq!(next.int("count"), Some(2));
        assert_eq!(next.child("next").unwrap().id, root.id);
    }

    #[test]
    fn damaged_streams_are_errors_not_panics() {
        let mut heap = Heap::default();
        heap.add(
            &NODE,
            vec![
                Field::Str("a".into()),
                Field::I32(1),
                Field::Null,
                Field::Null,
            ],
        );
        let good = write(&heap, &["Test"]);
        for end in 0..good.len() {
            assert!(parse(&good[..end]).is_err(), "{end}");
        }
        for at in 17..good.len() {
            for bad in [0u8, 7, 14, 0x7f, 0xff] {
                let mut copy = good.clone();
                copy[at] = bad;
                let _ = parse(&copy);
            }
        }
        // An array that claims far more slots than the limit.
        let mut huge = good[..17].to_vec();
        huge.push(16);
        huge.extend(5i32.to_le_bytes());
        huge.extend(i32::MAX.to_le_bytes());
        assert!(parse(&huge).is_err());
    }
}
