// Independent AXF codec. Vendor assemblies are used only by reference tests.
using System;
using System.IO;
using System.IO.Compression;
using System.Security.Cryptography;
using System.Text;
using System.Linq;
using System.Collections;
using System.Collections.Generic;
using O = System.Collections.Generic.Dictionary<string,object>;
namespace SketchRust.Compat {
 public sealed partial class ApexFile {
  const int Limit=32*1024*1024;
  static readonly Encoding Legacy=Encoding.GetEncoding(1252,EncoderFallback.ExceptionFallback,DecoderFallback.ExceptionFallback);
  static readonly byte[] FormatKey={237,92,128,46,82,20,96,145,245,105,27,241,209,6,66,6};
  public int Version=38;
  public int Measurement {get{return Version>=18?BodyPrefix[Version>=6?4:0]:5;}}
  int KeyOffset {get{return Version>=23?(Version>=6?4:0)+(Version>=18?1:0)+(Version>=19?1:0):-1;}}
  public byte[] Header, Metadata, BodyPrefix, Tail;
  public int ExtraSubjectOffset=-1;
  public Dictionary<string,string> Subject=new Dictionary<string,string>();
  public List<NativePage> Pages=new List<NativePage>();
  public static readonly string[] SubjectKeys={"FileNumber","Borrower","CaseNumber","PropertyAddress","City","County","State","ZipCode","LenderClient","LenderClientAddress","Appraiser","AppraiserAddress"};
  public sealed class NativePage {
   public List<NativeArea> Areas=new List<NativeArea>();public List<byte[]> Text=new List<byte[]>();
   public List<Ent> Entities=new List<Ent>();public List<Block> Blocks=new List<Block>();
   public byte[] Layout, Attributes; public int Key,VisibleLayers,AreaKey,TextKey;
  }
  public sealed class NativeArea {
   public string Code="UND",Description="",Name="",Subtract="";public byte Closed=1,Calc,Sign=1,LineStyle,Error,Draw;
   public double Factor=1,Perimeter,Height,Wall,StoredArea;public ushort Fill=65535,FillIndex,PreEntity;
   public int Color,Entity,NameIndex,SizeIndex,Key,SecondKey;public List<int> Subtracts=new List<int>();public byte[] Attributes=new byte[4],Extra=new byte[]{1,0,0,0,0,0};
  }
  public sealed class Ent {
   public short Type;public byte[] Prefix;public double[] Values=new double[0];public string Name="",Text="",Font="Arial";public short Flags;
   public List<Ent> Children=new List<Ent>();public ushort Label;public int Key;public byte[] Extras=new byte[0];
  }
  public sealed class Block {public string Name="",Layer="";public byte Position;public double X,Y;public List<Ent> Entities=new List<Ent>();public byte[] Tail;}
  sealed class R : BinaryReader {
   public int V;
   public R(byte[] bytes,int version=38):base(new MemoryStream(bytes)){V=version;}
   public byte[] Bytes(int n){if(n<0||n>Limit||n>BaseStream.Length-BaseStream.Position)throw new InvalidDataException("Invalid AXF record length");return ReadBytes(n);}
   public byte[] Slice(long start){long end=BaseStream.Position;BaseStream.Position=start;var bytes=Bytes(checked((int)(end-start)));return bytes;}
   public string Short(){return Legacy.GetString(Bytes(ReadByte())).TrimEnd('\0');}
   public string Long(){return Legacy.GetString(Bytes(ReadUInt16())).TrimEnd('\0');}
   public string Str(int token=-1){int t=token<0?ReadByte():token;int n;Encoding encoding;switch(t){case 4:case 6:n=ReadByte();encoding=Legacy;break;case 20:n=ReadInt32();encoding=new UTF8Encoding(false,true);break;case 18:n=checked(ReadInt32()*2);encoding=Encoding.Unicode;break;default:throw new InvalidDataException("Invalid AXF string token "+t);}return encoding.GetString(Bytes(n));}
   public string Layer(){switch(ReadByte()){case 253:return "S1";case 254:return "BYBLOCK";case 255:return "BYLAYER";case 0:return Short();default:return "";}}
   public byte[] Attr(){long start=BaseStream.Position;int n=ReadInt32();if(n<0||n>100000)throw new InvalidDataException("Invalid AXF attributes");for(int i=0;i<n;i++){Str();Str();if(ReadByte()!=0)throw new InvalidDataException("Invalid attribute end");}return Slice(start);}
   public byte[] Empty(){long start=BaseStream.Position;if(ReadUInt16()!=1)throw new InvalidDataException("Expected empty AXF list");if(V>=32)ReadInt32();return Slice(start);}
   public void Start(){if(ReadByte()!=1)throw new InvalidDataException("Invalid AXF list");}
   public int End(){return V>=32?ReadInt32():0;}
  }
  sealed class W : BinaryWriter {
   public int V;public W(Stream s,int v):base(s){V=v;}
   public void Fixed(string s,int n,bool pascal){var b=Legacy.GetBytes(s??"");if(b.Length>n)b=b.Take(n).ToArray();if(pascal)Write((byte)b.Length);Write(b);Write(new byte[n-b.Length]);}
   public void Str(string s){byte[] b=V>=35?Encoding.UTF8.GetBytes(s??""):Legacy.GetBytes(s??"");if(V<35&&b.Length>255)throw new InvalidDataException("Text exceeds the older AXF format limit");Write((byte)(V>=35?20:6));if(V>=35)Write(b.Length);else Write((byte)b.Length);Write(b);}
   public void Short(string s){var b=Legacy.GetBytes(s??"");if(b.Length>255)throw new InvalidDataException("AXF identifier too long");Write((byte)b.Length);Write(b);}
   public void Long(string s){var b=Legacy.GetBytes(s??"");if(b.Length>65535)throw new InvalidDataException("AXF text too long");Write((ushort)b.Length);Write(b);}
   public void Layer(string s){Write((byte)0);Short(s);}
   public void Empty(int key=0){Write((ushort)1);if(V>=32)Write(key);}
   public void Point(double x,double y){Write(x);Write(y);}
  }
  static byte[] Pack(Action<W> action,int version){using(var s=new MemoryStream()){using(var w=new W(s,version)){action(w);w.Flush();return s.ToArray();}}}
  static byte[] Inflate(byte[] bytes){using(var input=new MemoryStream(bytes))using(var zip=new DeflateStream(input,CompressionMode.Decompress))using(var output=new MemoryStream()){var block=new byte[8192];int n;while((n=zip.Read(block,0,block.Length))>0){if(output.Length+n>Limit)throw new InvalidDataException("AXF exceeds the expanded size limit");output.Write(block,0,n);}return output.ToArray();}}
  static byte[] Deflate(byte[] bytes){using(var output=new MemoryStream()){using(var zip=new DeflateStream(output,CompressionLevel.Optimal,true))zip.Write(bytes,0,bytes.Length);return output.ToArray();}}
  public static ApexFile Read(byte[] bytes){
   if(bytes.Length==0||bytes.Length>Limit)throw new InvalidDataException("Invalid AXF file size");
   var f=new ApexFile();var outer=bytes[0]==7?bytes:Inflate(bytes);
   using(var r=new R(outer)){
    f.Header=r.Bytes(18);if(f.Header[0]!=7||Encoding.ASCII.GetString(f.Header,1,7)!="APX4WIN"||f.Header[11]!=64||f.Header[16]!=1)throw new InvalidDataException("Not an Apex sketch file");
    f.Version=f.Header[17];if(f.Version<1||f.Version>38)throw new NotSupportedException("Unsupported AXF version "+f.Version);f.Header=f.Header.Concat(r.Bytes(36+(f.Version>=21?11:0)+(f.Version>=22?42:0))).ToArray();if(f.Version==1&&BitConverter.ToInt32(f.Header,36)>=(new DateTime(1997,2,1)-new DateTime(1600,1,1)).Days)f.Version=2;r.V=f.Version;
    foreach(string key in SubjectKeys)f.Subject[key]=r.Str();long meta=r.BaseStream.Position;if(f.Version>=32)r.ReadInt32();
    foreach(int list in new[]{0,1}){int count=r.ReadUInt16();for(int i=0;i<count;i++)r.Str();if(f.Version>=32)r.ReadInt32();}
    if(f.Version>=35){uint count=r.ReadUInt32();if(count>100000)throw new InvalidDataException("Too many AXF comments");for(int i=0;i<count;i++){r.Str();r.Str();r.ReadInt32();r.Str();}f.ExtraSubjectOffset=checked((int)(r.BaseStream.Position-meta));uint extra=r.ReadUInt32();if(extra>100000)throw new InvalidDataException("Too many subject fields");for(int i=0;i<extra;i++){string name=r.Str();f.Subject[name]=r.Str();}}
    f.Metadata=r.Slice(meta);
    byte[] body;
    if(f.Version>=34){var iv=r.Bytes(16);var encrypted=r.Bytes(checked((int)(r.BaseStream.Length-r.BaseStream.Position)));using(var aes=Aes.Create()){aes.Key=FormatKey;aes.IV=iv;var plain=aes.CreateDecryptor().TransformFinalBlock(encrypted,0,encrypted.Length);if(plain.Length<8)throw new InvalidDataException("Truncated AXF payload");long size=0;for(int i=0;i<8;i++)size=checked(size*256+plain[i]);if(size<0||size>Limit||size!=plain.Length-8)throw new InvalidDataException("Invalid AXF payload length");body=Inflate(plain.Skip(8).ToArray());}}
    else body=r.Bytes(checked((int)(r.BaseStream.Length-r.BaseStream.Position)));
    using(var b=new R(body,f.Version)){
     f.BodyPrefix=b.Bytes((f.Version>=6?4:0)+(f.Version>=18?1:0)+(f.Version>=19?1:0)+(f.Version>=23?4:0)+(f.Version>=28?20:0));if(f.Measurement!=5&&f.Measurement!=2)throw new InvalidDataException("Invalid AXF measurement unit");
     b.Start();int token;while((token=b.ReadByte())!=0){if(token!=1)throw new InvalidDataException("Invalid AXF page");if(f.Pages.Count>=256)throw new InvalidDataException("Too many AXF pages");f.Pages.Add(ReadPage(b));}b.End();f.Tail=b.Bytes(checked((int)(b.BaseStream.Length-b.BaseStream.Position)));
    }
   }return f;
  }

  static NativePage ReadPage(R r){var p=new NativePage();int token;while((token=r.ReadByte())!=0){var area=ReadArea(r,token);if(r.V<23)area.Key=p.Areas.Count;p.Areas.Add(area);}p.AreaKey=r.End();r.Start();while((token=r.ReadByte())!=0){long begin=r.BaseStream.Position;ReadText(r,token);var raw=r.Slice(begin);p.Text.Add(new[]{(byte)token}.Concat(raw).ToArray());}p.TextKey=r.End();long start=r.BaseStream.Position;r.Empty();if(r.V>=2)r.Bytes(24);if(r.V>=14)r.Bytes(12);if(r.V>=15)r.Bytes(4);if(r.V>=25)r.Bytes(4);p.Layout=r.Slice(start);p.Entities=ReadEntities(r);p.Blocks=ReadBlocks(r);if(r.V>=32)p.Key=r.ReadInt32();if(r.V>=33)p.VisibleLayers=r.ReadInt32();p.Attributes=r.V>=37?r.Attr():new byte[0];return p;}
  static readonly string[] EarlyCodes={"GLA1","GLA2","GLA3","GLA4","GLA5","GLA6","GLA7","GLA8","GLA9","GLA0","GBA1","GBA2","GBA3","GBA4","GBA5","GBA6","GBA7","GBA8","GBA9","GBA0","BSMT","P/P","GAR","LAND","SITE","OTH","NCA","FF","WIN"};
  static NativeArea ReadArea(R r,int token){
   var a=new NativeArea();
   if(r.V<=6){if(token<1||token>EarlyCodes.Length||r.ReadByte()!=0)throw new InvalidDataException("Invalid early native area code");a.Code=EarlyCodes[token-1];}
   else if(r.V==7){if(token>5)throw new InvalidDataException("Invalid fixed area code");a.Code=Legacy.GetString(r.Bytes(5),0,token).TrimEnd('\0');}
   else a.Code=r.Str(token);
   if(r.V>=13)a.Description=r.Str();a.Closed=r.ReadByte();a.Factor=r.ReadDouble();if(r.V<26)a.StoredArea=r.ReadDouble();a.Perimeter=r.ReadDouble();a.Height=r.ReadDouble();a.Wall=r.ReadDouble();a.Fill=r.ReadUInt16();if(r.V<=10&&a.Fill==0)a.Fill=65535;if(r.V>=12)a.Color=r.ReadInt32();a.Calc=r.ReadByte();a.Sign=r.ReadByte();a.LineStyle=r.ReadByte();a.Name=r.Str();
   if(r.V>=22){a.Subtract=r.Str();if(r.V<=23){int index=r.ReadInt16();if(index<0)throw new InvalidDataException("Invalid subtraction index");if(index!=0)a.Subtracts.Add(index-1);}else{if(r.ReadByte()!=1)throw new InvalidDataException("Invalid subtraction list");int x;while((x=r.ReadByte())!=0){x|=r.ReadByte()<<8;x|=r.ReadUInt16()<<16;a.Subtracts.Add(x-1);if(a.Subtracts.Count>65535)throw new InvalidDataException("Too many subtractions");}}}
   a.Entity=r.ReadUInt16();if(r.V>=3)a.FillIndex=r.ReadUInt16();if(r.V>=10)a.PreEntity=r.ReadUInt16();if(r.V>=16){a.NameIndex=r.ReadInt32();a.SizeIndex=r.ReadInt32();}if(r.V>=23)a.Key=r.ReadInt32();if(r.V>=27)a.Error=r.ReadByte();if(r.V>=29)a.Draw=r.ReadByte();if(r.V>=32)a.SecondKey=r.ReadInt32();if(r.V>=35)a.Attributes=r.Attr();a.Extra=r.Empty();return a;
  }

  static O ReadText(R r,int token){int kind=r.V>=9?token:1;if(kind!=1&&kind!=2)throw new InvalidDataException("Invalid text class");int stringToken=r.V>=9?r.ReadByte():token;string text=r.Str(stringToken);int color=r.ReadInt32();double x=r.ReadDouble(),y=r.ReadDouble();byte[] font=r.Bytes(r.V>=21?60:50);byte charset=font[r.V>=21?23:13];if((stringToken==4||stringToken==6)&&charset!=0){int code=CharsetCode(charset);text=Encoding.GetEncoding(code).GetString(Legacy.GetBytes(text));}double height=r.ReadDouble(),width=r.ReadDouble();r.ReadByte();double rotation=r.ReadDouble();if(r.V>=30)r.Str();if(r.V>=4&&r.V<=8)r.ReadByte();int key=r.V>=32?r.ReadInt32():0;if(r.V>=9&&kind==2)r.ReadByte();return Obj("id",key,"x",x,"y",y,"text",text,"fontSize",height*20,"rotation",-rotation,"color",Color(color),"fontFamily",Legacy.GetString(font,r.V>=21?28:18,32).TrimEnd('\0'),"bold",(r.V>=21?BitConverter.ToInt32(font,16):BitConverter.ToInt16(font,8))>=700,"italic",font[r.V>=21?20:10]!=0);}
  static List<Ent> ReadEntities(R r,int depth=0){if(depth>32)throw new InvalidDataException("Nested CAD entities exceed limit");var list=new List<Ent>();if(r.ReadByte()!=0) return list;int count=r.ReadInt16();if(count<0)throw new InvalidDataException("Invalid CAD count");for(int i=0;i<count;i++)list.Add(ReadEntity(r,depth));return list;}
  static Ent ReadEntity(R r,int depth){var e=new Ent();e.Type=r.ReadInt16();long start=r.BaseStream.Position;r.Bytes(40+(r.V>=33?4:2)+16);r.Layer();r.Layer();e.Prefix=r.Slice(start);
   int doubles;switch(e.Type){case 1:doubles=4;break;case 2:doubles=4;break;case 3:doubles=3;break;case 4:doubles=5;break;case 35:doubles=8;break;case 14:doubles=6;break;case 13:e.Flags=r.ReadInt16();doubles=2;break;case 23:e.Name=r.Short();doubles=5;break;case 7:doubles=3;break;default:throw new NotSupportedException("Unknown native CAD entity "+e.Type);}
   e.Values=new double[doubles];for(int j=0;j<doubles;j++)e.Values[j]=r.ReadDouble();
   if(e.Type==1)e.Extras=r.Bytes(r.V>=21?4:2);
   if(e.Type==13)e.Children=ReadEntities(r,depth+1);
   if(e.Type==23){int marker=r.ReadByte();if(marker==0&&r.ReadInt16()!=0)throw new InvalidDataException("Unsupported insert attributes");}
   if(e.Type==7){e.Text=r.Long();e.Extras=r.Bytes(24);e.Font=r.Short();if(r.V>=38)e.Extras=e.Extras.Concat(r.Bytes(3)).ToArray();}
   e.Label=r.ReadUInt16();if(r.V<=2&&e.Label!=0)e.Label++;if(r.V>=32)e.Key=r.ReadInt32();return e;
  }
  static List<Block> ReadBlocks(R r){var list=new List<Block>();if(r.ReadByte()!=0)return list;int count=r.ReadInt16();if(count<0)throw new InvalidDataException("Invalid block count");for(int i=0;i<count;i++){if(r.ReadInt16()!=9)throw new InvalidDataException("Invalid CAD block");var b=new Block();b.Name=r.Short();if(r.ReadUInt16()!=262)throw new InvalidDataException("Invalid block style");b.Position=r.ReadByte();b.Layer=r.Layer();b.X=r.ReadDouble();b.Y=r.ReadDouble();b.Entities=ReadEntities(r);b.Tail=r.Bytes(r.V>=38?21:r.V>=32?4:0);list.Add(b);}return list;}

  static void WriteArea(W w,NativeArea a){
   if(w.V<=6){int index=Array.IndexOf(EarlyCodes,a.Code);if(index<0)throw new InvalidDataException("Area code is not supported in the early native format");w.Write(checked((ushort)(index+1)));}else if(w.V==7)w.Fixed(a.Code,5,true);else w.Str(a.Code);
   if(w.V>=13)w.Str(a.Description);w.Write(a.Closed);w.Write(a.Factor);if(w.V<26)w.Write(a.StoredArea);w.Write(a.Perimeter);w.Write(a.Height);w.Write(a.Wall);w.Write(w.V<=10&&a.Fill==65535?(ushort)0:a.Fill);if(w.V>=12)w.Write(a.Color);w.Write(a.Calc);w.Write(a.Sign);w.Write(a.LineStyle);w.Str(a.Name);
   if(w.V>=22){w.Str(a.Subtract);if(w.V<=23){if(a.Subtracts.Count>1)throw new NotSupportedException("This early native format supports one subtraction only");w.Write(checked((short)(a.Subtracts.Count==0?0:a.Subtracts[0]+1)));}else{w.Write((byte)1);foreach(int id in a.Subtracts){if(((id+1)&255)==0)throw new InvalidDataException("Unrepresentable subtraction ID");w.Write(id+1);}w.Write((byte)0);}}
   w.Write(checked((ushort)a.Entity));if(w.V>=3)w.Write(a.FillIndex);if(w.V>=10)w.Write(a.PreEntity);if(w.V>=16){w.Write(a.NameIndex);w.Write(a.SizeIndex);}if(w.V>=23)w.Write(a.Key);if(w.V>=27)w.Write(a.Error);if(w.V>=29)w.Write(a.Draw);if(w.V>=32)w.Write(a.SecondKey);if(w.V>=35)w.Write(a.Attributes);if(a.Extra.Length==(w.V>=32?6:2))w.Write(a.Extra);else w.Empty();
  }
  static void WriteEntities(W w,List<Ent> entities){w.Write((byte)0);w.Write(checked((short)entities.Count));foreach(var e in entities){w.Write(e.Type);w.Write(e.Prefix);if(e.Type==13)w.Write(e.Flags);if(e.Type==23)w.Short(e.Name);foreach(double value in e.Values)w.Write(value);if(e.Type==13)WriteEntities(w,e.Children);if(e.Type==1)w.Write(e.Extras);if(e.Type==23)w.Write((byte)1);if(e.Type==7){w.Long(e.Text);w.Write(e.Extras.Take(24).ToArray());w.Short(e.Font);if(w.V>=38)w.Write(e.Extras.Skip(24).Take(3).ToArray());}w.Write((ushort)(w.V<=2&&e.Label!=0?e.Label-1:e.Label));if(w.V>=32)w.Write(e.Key);}}
  static void WriteBlocks(W w,List<Block> blocks){w.Write((byte)0);w.Write(checked((short)blocks.Count));foreach(var b in blocks){w.Write((short)9);w.Short(b.Name);w.Write((ushort)262);w.Write(b.Position);w.Layer(b.Layer);w.Point(b.X,b.Y);WriteEntities(w,b.Entities);w.Write(b.Tail);}}
  public byte[] Write(){
   var body=Pack(w=>{w.Write(BodyPrefix);w.Write((byte)1);foreach(var p in Pages){w.Write((byte)1);foreach(var a in p.Areas)WriteArea(w,a);w.Write((byte)0);if(Version>=32)w.Write(p.AreaKey);w.Write((byte)1);foreach(var t in p.Text)w.Write(t);w.Write((byte)0);if(Version>=32)w.Write(p.TextKey);w.Write(p.Layout);WriteEntities(w,p.Entities);WriteBlocks(w,p.Blocks);if(Version>=32)w.Write(p.Key);if(Version>=33)w.Write(p.VisibleLayers);if(Version>=37)w.Write(p.Attributes);}w.Write((byte)0);if(Version>=32)w.Write(0);w.Write(Tail);},Version);
   var full=Pack(w=>{w.Write(Header);foreach(var key in SubjectKeys){string value;w.Str(Subject.TryGetValue(key,out value)?value:"");}if(Version>=35){w.Write(Metadata.Take(ExtraSubjectOffset).ToArray());var extra=Subject.Where(pair=>!SubjectKeys.Contains(pair.Key)).ToArray();w.Write(extra.Length);foreach(var pair in extra){w.Str(pair.Key);w.Str(pair.Value);}}else {if(Subject.Keys.Any(key=>!SubjectKeys.Contains(key)))throw new NotSupportedException("Subject tags require native format version 35 or newer");w.Write(Metadata);}if(Version<34)w.Write(body);else{var compressed=Deflate(body);byte[] plain=new byte[8+compressed.Length];long size=compressed.Length;for(int i=7;i>=0;i--){plain[i]=(byte)size;size>>=8;}compressed.CopyTo(plain,8);using(var aes=Aes.Create()){aes.Key=FormatKey;aes.GenerateIV();w.Write(aes.IV);w.Write(aes.CreateEncryptor().TransformFinalBlock(plain,0,plain.Length));}}},Version);return Deflate(full);
  }

  public static ApexFile New(int version=38){
   if(version<1||version>38)throw new ArgumentOutOfRangeException("version");var f=new ApexFile{Version=version};
   f.Header=Pack(w=>{w.Fixed("APX4WIN",10,true);w.Write((byte)64);w.Write(version>=22?272:version>=21?16:0);w.Write((byte)1);w.Write((byte)version);for(int i=0;i<2;i++){var now=version==1?new DateTime(1996,1,1):DateTime.Now;w.Write((now.Date-new DateTime(1600,1,1)).Days);w.Write((int)now.TimeOfDay.TotalSeconds);w.Fixed("SketchRust",10,false);}if(version>=21)w.Fixed("SketchRust",10,true);if(version>=22){w.Fixed("0.1.29",20,true);w.Fixed("0.1.8",20,true);}},version);
   f.Metadata=Pack(w=>{if(version>=32)w.Write(0);w.Write((ushort)0);if(version>=32)w.Write(0);w.Write((ushort)33);for(int i=0;i<33;i++)w.Str(i%11==0?"Comment Table "+(i/11+1):"");if(version>=32)w.Write(0);if(version>=35){w.Write(0);w.Write(0);}},version);f.ExtraSubjectOffset=version>=35?f.Metadata.Length-4:-1;
   f.BodyPrefix=Pack(w=>{if(version>=6)w.Write(257);if(version>=18)w.Write((byte)5);if(version>=19)w.Write((byte)0);if(version>=23)w.Write(1);if(version>=28)w.Write(new byte[20]);},version);
   f.Tail=version<36?new byte[0]:Pack(w=>{w.Write((byte)2);w.Write((sbyte)2);w.Write((byte)2);w.Write((sbyte)2);w.Str(" sf");w.Str("'");w.Write((byte)8);w.Write((byte)2);w.Write((sbyte)2);w.Write((byte)8);w.Write((byte)21);w.Write(0.0);w.Write((byte)8);w.Write(0);},version);return f;
  }
  public static O Obj(params object[] pairs){var o=new O();for(int i=0;i<pairs.Length;i+=2)o[(string)pairs[i]]=pairs[i+1];return o;}
  public static object Get(O o,string key,object fallback=null){object v;return o!=null&&o.TryGetValue(key,out v)&&v!=null?v:fallback;}
  public static double Num(O o,string key,double fallback=0){var v=Get(o,key);return v==null?fallback:Convert.ToDouble(v,System.Globalization.CultureInfo.InvariantCulture);}
  public static string Str(O o,string key,string fallback=""){return Convert.ToString(Get(o,key,fallback),System.Globalization.CultureInfo.InvariantCulture);}
  public static O Map(object value){return value as O??new O();}
  public static IEnumerable<O> Items(O o,string key){return ((Get(o,key) as IEnumerable)??new object[0]).Cast<object>().Select(Map);}
  static O Pt(double x,double y){return Obj("x",x,"y",y);}
  static string Color(int c){return "#"+((c&255)<<16|(c&65280)|((c>>16)&255)).ToString("x6");}
  static byte[] Common(int version,int color=0,string layer="0"){return Pack(w=>{w.Write(new byte[40]);if(version>=33)w.Write(color);else w.Write(PaletteIndex(color));var extrusion=new byte[16];extrusion[4]=255;w.Write(extrusion);w.Layer(layer);w.Layer("CONTINUOUS");},version);}
  static Ent Entity(short type,int key,int version,params double[] values){return new Ent{Type=type,Key=key,Prefix=Common(version),Values=values};}


  static O Tags(byte[] raw,int version){var result=new O();using(var r=new R(raw,version)){int n=r.ReadInt32();for(int i=0;i<n;i++){string name=r.Str(),value=r.Str();r.ReadByte();if(name.StartsWith("#Byte[]#"))continue;if(value.StartsWith("#D")||value.StartsWith("#S"))value=value.Substring(2);result[name]=value;}}return result;}
  static byte[] StoreTags(byte[] raw,O values,int version){
   var records=new List<byte[]>();var remaining=new O(values);remaining.Remove("apex.description");remaining.Remove("apex.averageWallHeight");remaining.Remove("apex.calculation");
   if(version<35){if(remaining.Count!=0)throw new NotSupportedException("Area tags require native version 35 or newer");return raw;}
   using(var r=new R(raw,version)){int n=r.ReadInt32();for(int i=0;i<n;i++){long start=r.BaseStream.Position;string name=r.Str(),value=r.Str();r.ReadByte();byte[] record=r.Slice(start);if(name.StartsWith("#Byte[]#")){records.Add(record);continue;}object current;
    if(remaining.TryGetValue(name,out current)){string plain=(value.StartsWith("#D")||value.StartsWith("#S"))?value.Substring(2):value;if(Convert.ToString(current)==plain){records.Add(record);remaining.Remove(name);}}
   }}
   foreach(var pair in remaining){string value=Convert.ToString(pair.Value);if(value.StartsWith("#"))throw new InvalidDataException("Native Apex user attribute values cannot start with #");records.Add(Pack(w=>{w.Str(pair.Key);w.Str(value);w.Write((byte)0);},version));}
   return Pack(w=>{w.Write(records.Count);foreach(var record in records)w.Write(record);},version);
  }

  static List<O> GeoPoints(NativePage page,int version){
   var result=new List<O>();if(version<37)return result;var tags=Tags(page.Attributes,version);
   for(int i=0;i<2;i++){string value=Str(tags,"GEOREF"+i);if(value.Length==0)break;var parts=value.Split(new[]{'|'},5);if(parts.Length!=5||parts[0]!=""||parts[1]!=i.ToString())throw new InvalidDataException("Invalid native georeference point");result.Add(Obj("x",double.Parse(parts[2],System.Globalization.CultureInfo.InvariantCulture),"y",double.Parse(parts[3],System.Globalization.CultureInfo.InvariantCulture),"coordinates",parts[4]));}return result;
  }
  static string GeoSystem(NativePage page,int version){if(version<37)return "";string value=Str(Tags(page.Attributes,version),"GEOCRS");return value.StartsWith("WKT:")?value.Substring(4):"";}
  static byte[] StoreGeo(NativePage page,O model,int version){
   var points=Items(model,"geoReferencePoints").ToArray();string system=Str(model,"geoReferenceCoordinateSystem");if(points.Length>2)throw new InvalidDataException("Apex supports two georeference points");
   if(version<37){if(points.Length!=0||system.Length!=0)throw new NotSupportedException("Georeference data requires native format version 37 or newer");return page.Attributes;}
   var records=new List<byte[]>();var added=new List<byte[]>();
   for(int i=0;i<points.Length;i++){string value="|"+i+"|"+Num(points[i],"x").ToString("R",System.Globalization.CultureInfo.InvariantCulture)+"|"+Num(points[i],"y").ToString("R",System.Globalization.CultureInfo.InvariantCulture)+"|"+Str(points[i],"coordinates");string name="GEOREF"+i;added.Add(Pack(w=>{w.Str(name);w.Str(value);w.Write((byte)0);},version));}
   if(system.Length!=0)added.Add(Pack(w=>{w.Str("GEOCRS");w.Str("WKT:"+system);w.Write((byte)0);},version));
   using(var r=new R(page.Attributes,version)){int n=r.ReadInt32();for(int i=0;i<n;i++){long start=r.BaseStream.Position;string name=r.Str();r.Str();r.ReadByte();var raw=r.Slice(start);if(name=="GEOCRS"||name.StartsWith("GEOREF"))continue;if(name=="_#U"){records.AddRange(added);added.Clear();}records.Add(raw);}}
   records.AddRange(added);return Pack(w=>{w.Write(records.Count);foreach(var record in records)w.Write(record);},version);
  }
  static O Inch(double x,double y){return Pt(x*12,y*12);}
  static double Hypot(double x,double y){return Math.Sqrt(x*x+y*y);}
  static List<O> Geometry(IEnumerable<Ent> entities,double ox=0,double oy=0){
   var result=new List<O>();foreach(var e in entities){var v=e.Values;O g;
    switch(e.Type){
     case 2:g=Obj("entityType","LINE","startPoint",Inch(v[0]-ox,v[1]-oy),"endPoint",Inch(v[2]-ox,v[3]-oy));break;
     case 3:g=Obj("entityType","ELLIPSE","center",Inch(v[0]-ox,v[1]-oy),"axisPoint",Inch(v[2],0),"ratio",1);break;
     case 4:g=Obj("entityType","ARC","center",Inch(v[0]-ox,v[1]-oy),"axisPoint",Inch(v[2],0),"ratio",1,"startAngle",v[3],"endAngle",v[4]);break;
     case 35:g=Obj("entityType","ELLIPSE","center",Inch(v[0]-ox,v[1]-oy),"axisPoint",Inch(v[3],v[4]),"ratio",v[5]/Math.Max(1e-12,Hypot(v[3],v[4])),"startAngle",v[6],"endAngle",v[7]);break;
     case 13:g=Obj("entityType","POLYLINE","closed",(e.Flags&1)!=0,"vertices",e.Children.Select(q=>Obj("x",(q.Values[0]-ox)*12,"y",(q.Values[1]-oy)*12,"b",q.Values[5])).ToArray());break;
     case 7:g=Obj("entityType","TEXT","insertPoint",Inch(v[0]-ox,v[1]-oy),"fontSize",v[2]*12,"textString",e.Text,"rotation",BitConverter.ToDouble(e.Extras,0),"alignment","LEFT BOTTOM");break;
     default:throw new NotSupportedException("Unsupported block CAD entity "+e.Type+"; the native file has not been changed");
    }
    int color=e.Prefix.Length>=44?BitConverter.ToInt32(e.Prefix,40):0;g["strokeColor"]=color;result.Add(g);
   }return result;
  }
  static O Definition(string name,List<O> geometry){
   var points=new List<O>();foreach(var g in geometry){foreach(string field in new[]{"startPoint","endPoint","center","insertPoint"})if(Get(g,field)!=null)points.Add(Map(Get(g,field)));points.AddRange(Items(g,"vertices"));}
   foreach(var g in geometry){
    string type=Str(g,"entityType");if(type=="ARC"||type=="ELLIPSE"){var c=Map(Get(g,"center"));var a=Map(Get(g,"axisPoint"));double ax=Num(a,"x"),ay=Num(a,"y"),ratio=Num(g,"ratio",1);double rx=Hypot(ax,-ay*ratio),ry=Hypot(ay,ax*ratio);points.Add(Pt(Num(c,"x")-rx,Num(c,"y")-ry));points.Add(Pt(Num(c,"x")+rx,Num(c,"y")+ry));}
    if(type=="TEXT"){var c=Map(Get(g,"insertPoint",Get(g,"insertionPoint")));double size=Num(g,"fontSize",12),radius=Hypot(Str(g,"textString").Length*size,size);points.Add(Pt(Num(c,"x")-radius,Num(c,"y")-radius));points.Add(Pt(Num(c,"x")+radius,Num(c,"y")+radius));}
    if(type=="POLYLINE"){var vertices=Items(g,"vertices").ToArray();int count=Object.Equals(Get(g,"closed"),true)?vertices.Length:vertices.Length-1;for(int j=0;j<count;j++){var a=vertices[j];var b=vertices[(j+1)%vertices.Length];double bulge=Num(a,"b");if(Math.Abs(bulge)<1e-12)continue;double dx=Num(b,"x")-Num(a,"x"),dy=Num(b,"y")-Num(a,"y"),cx=(Num(a,"x")+Num(b,"x"))/2-dy*(1-bulge*bulge)/(4*bulge),cy=(Num(a,"y")+Num(b,"y"))/2+dx*(1-bulge*bulge)/(4*bulge);double radius=Hypot(Num(a,"x")-cx,Num(a,"y")-cy),start=Math.Atan2(Num(a,"y")-cy,Num(a,"x")-cx),sweep=4*Math.Atan(bulge);for(int k=0;k<4;k++){double t=k*Math.PI/2,delta=sweep>0?t-start:start-t;delta=(delta%(2*Math.PI)+2*Math.PI)%(2*Math.PI);if(delta<=Math.Abs(sweep)+1e-9)points.Add(Pt(cx+radius*Math.Cos(t),cy+radius*Math.Sin(t)));}}}
   }
   double minX=points.Count==0?-6:points.Min(p=>Num(p,"x")),maxX=points.Count==0?6:points.Max(p=>Num(p,"x"));
   double minY=points.Count==0?-6:points.Min(p=>Num(p,"y")),maxY=points.Count==0?6:points.Max(p=>Num(p,"y"));
   return Obj("id",name,"name",name,"label",name,"category","native-apex","categoryName","Native Apex","units","INCHES","wallSnap","NONE","geometry",geometry,"bounds",Obj("minX",minX,"minY",minY,"maxX",maxX,"maxY",maxY));
  }
  static List<Ent> SymbolEntities(O definition,int version,ref int next){
   var result=new List<Ent>();foreach(var g in Items(definition,"geometry")){if(Object.Equals(Get(g,"nativePoint"),true))continue;
    var a=Map(Get(g,"startPoint"));var b=Map(Get(g,"endPoint"));var c=Map(Get(g,"center"));var axis=Map(Get(g,"axisPoint"));double radius=Hypot(Num(axis,"x"),Num(axis,"y"))/12;Ent e;
    switch(Str(g,"entityType")){
     case "LINE":e=Entity(2,next++,version,Num(a,"x")/12,Num(a,"y")/12,Num(b,"x")/12,Num(b,"y")/12);break;
     case "POLYLINE":e=Entity(13,next++,version,0,0);e.Flags=(short)(Object.Equals(Get(g,"closed"),true)?1:0);foreach(var v in Items(g,"vertices"))e.Children.Add(Entity(14,next++,version,Num(v,"x")/12,Num(v,"y")/12,0,0,0,Num(v,"b")));break;
     case "ARC":e=Entity(4,next++,version,Num(c,"x")/12,Num(c,"y")/12,radius,Num(g,"startAngle"),Num(g,"endAngle"));break;
     case "ELLIPSE":e=Entity(35,next++,version,Num(c,"x")/12,Num(c,"y")/12,0,Num(axis,"x")/12,Num(axis,"y")/12,radius*Num(g,"ratio",1),Num(g,"startAngle"),Num(g,"endAngle",360));break;
     case "TEXT":var point=Map(Get(g,"insertPoint",Get(g,"insertionPoint")));e=Entity(7,next++,version,Num(point,"x")/12,Num(point,"y")/12,Num(g,"fontSize",12)/12);e.Text=Str(g,"textString");e.Extras=Pack(w=>{w.Write(Num(g,"rotation"));w.Write(1.0);w.Write(0.0);if(version>=38)w.Write(new byte[]{0,1,0});},version);break;
     default:throw new NotSupportedException("Unsupported symbol geometry "+Str(g,"entityType"));
    }
    e.Prefix=Common(version,checked((int)Num(g,"strokeColor")));int offset=40+(version>=33?4:2);e.Prefix[offset+4]=(byte)Num(g,"strokeAlpha",255);if(e.Type==7){BitConverter.GetBytes(checked((int)Num(g,"fillColor",Num(g,"strokeColor")))).CopyTo(e.Prefix,offset+8);e.Prefix[offset+11]=255;}else if(Get(g,"fillColor")!=null&&Num(g,"fillColor")>=0){BitConverter.GetBytes(checked((int)Num(g,"fillColor"))).CopyTo(e.Prefix,offset+8);e.Prefix[offset+11]=255;}result.Add(e);
   }return result;
  }

  static List<Ent> BoundaryVertices(Ent boundary){
   if((boundary.Flags&1)==0)throw new NotSupportedException("This native file contains an unfinished area boundary. Close it in its source application before importing; the file has not been changed");
   if(boundary.Children.Count!=2)return boundary.Children;
   var list=new List<Ent>();for(int i=0;i<2;i++){var a=boundary.Children[i];var b=boundary.Children[(i+1)%2];double bulge=a.Values[5];if(Math.Abs(bulge)<1e-12){list.Add(a);continue;}
    double dx=b.Values[0]-a.Values[0],dy=b.Values[1]-a.Values[1],cx=(a.Values[0]+b.Values[0])/2-dy*(1-bulge*bulge)/(4*bulge),cy=(a.Values[1]+b.Values[1])/2+dx*(1-bulge*bulge)/(4*bulge),angle=2*Math.Atan(bulge),x=a.Values[0]-cx,y=a.Values[1]-cy;
    var start=new Ent{Type=14,Key=a.Key,Values=(double[])a.Values.Clone()};start.Values[5]=Math.Tan(Math.Atan(bulge)/2);list.Add(start);
    list.Add(new Ent{Type=14,Key=-1,Values=new[]{cx+x*Math.Cos(angle)-y*Math.Sin(angle),cy+x*Math.Sin(angle)+y*Math.Cos(angle),0,0,0,start.Values[5]}});
   }return list;
  }
  public O ToModel(){
   int next=KeyOffset>=0?Math.Max(1,BitConverter.ToInt32(BodyPrefix,KeyOffset)+1):1;var pages=new List<O>();var custom=new O();var used=new HashSet<int>();var reserved=new HashSet<int>();
   foreach(var page in Pages){var map=new Dictionary<int,int>();foreach(var area in page.Areas){if(map.ContainsKey(area.Key))throw new InvalidDataException("Duplicate native area key");int value=area.Key+1;if(value<1||reserved.Contains(value)){while(reserved.Contains(next))next++;value=next++;}map[area.Key]=value-1;reserved.Add(value);next=Math.Max(next,value+1);}foreach(var area in page.Areas){area.Key=map[area.Key];area.Subtracts=area.Subtracts.Select(key=>{int value;if(!map.TryGetValue(key,out value))throw new InvalidDataException("Missing native subtraction area");return value;}).ToList();}}
   Func<int,int> id=old=>{int value=old;if(value<1||used.Contains(value)||reserved.Contains(value)){while(used.Contains(next)||reserved.Contains(next))next++;value=next++;}used.Add(value);next=Math.Max(next,value+1);return value;};
   if(Measurement==2){var feet=Read(Write());feet.ScaleUnits(1/0.3048);feet.BodyPrefix[4]=5;var converted=feet.ToModel();Map(Get(converted,"settings"))["apexDocument"]=Obj("units",2);return converted;}
   foreach(var page in Pages){
    foreach(var area in page.Areas)NativeNet(page,area,new HashSet<int>());
    foreach(var parent in page.Areas)foreach(int childId in parent.Subtracts){var child=page.Areas.FirstOrDefault(area=>area.Key==childId);if(child==null)throw new InvalidDataException("Missing native subtraction area");if(child.Sign==0||parent.Sign==0)throw new InvalidDataException("Invalid signed native subtraction");}
    var lines=new List<O>();var polygons=new List<O>();var labels=new List<O>();var symbols=new List<O>();var assigned=new HashSet<int>();var drafts=ReadDrafts(page);
    foreach(var area in page.Areas){if(area.Entity<0||area.Entity>=page.Entities.Count||page.Entities[area.Entity].Type!=13)throw new InvalidDataException("Invalid native area boundary");var boundary=page.Entities[area.Entity];bool closed=area.Closed!=0&&(boundary.Flags&1)!=0;var vertices=closed?BoundaryVertices(boundary):boundary.Children;if(vertices.Any(v=>v.Type!=14))throw new InvalidDataException("Invalid native polyline vertices");int areaId=area.Key+1;used.Add(areaId);var points=new List<O>();var edges=new List<O>();for(int i=0;i<vertices.Count;i++){var v=vertices[i];var end=vertices[(i+1)%vertices.Count];int lineId=!closed&&lines.Any(l=>Num(l,"id")==v.Key+1&&Math.Abs(Num(Map(Get(l,"start")),"x")-v.Values[0])<1e-7&&Math.Abs(Num(Map(Get(l,"start")),"y")-v.Values[1])<1e-7)?v.Key+1:id(v.Key+1);v.Key=lineId-1;points.Add(Pt(v.Values[0],v.Values[1]));if((closed||i+1<vertices.Count)&&!lines.Any(l=>Num(l,"id")==lineId))lines.Add(Obj("id",lineId,"start",Pt(v.Values[0],v.Values[1]),"end",Pt(end.Values[0],end.Values[1]),"type","exterior","bowAngle",-4*Math.Atan(v.Values[5])*180/Math.PI));edges.Add(Obj("lineId",lineId));}assigned.Add(area.Entity);var areaModel=Obj("id",areaId,"points",points,"edges",edges,"type",area.Code,"name",area.Name,"factorOverride",Math.Abs(area.Factor)*(area.Sign==0?-1:1),"subtractMode","never","autoPost",Obj("name",area.NameIndex>0,"calcs",area.SizeIndex>0,"dims",boundary.Children.Any(v=>v.Label!=0)),"attributes",AreaTags(area));if(closed)polygons.Add(areaModel);else if(!drafts.Any(d=>Num(Map(Get(d,"original")),"id")==areaId||(Version<23&&Str(Map(Get(d,"original")),"name")==area.Name&&Str(Map(Get(d,"original")),"type")==area.Code)))drafts.Add(Obj("original",areaModel,"firstReplacementId",Math.Max(next,areaId+1)));}
    foreach(var e in page.Entities.Select((e,i)=>new{e,i})){if(assigned.Contains(e.i))continue;if(e.e.Type==2){lines.Add(Obj("id",id(e.e.Key+1),"start",Pt(e.e.Values[0],e.e.Values[1]),"end",Pt(e.e.Values[2],e.e.Values[3]),"type","interior","color",Color(EntityColor(e.e,Version))));}else if(e.e.Type==4){
     var v=e.e.Values;double sweep=(v[4]-v[3])*Math.PI/180%(2*Math.PI);if(sweep<=0)sweep+=2*Math.PI;int pieces=sweep>359*Math.PI/180?2:1;
     for(int part=0;part<pieces;part++){double start=v[3]*Math.PI/180+sweep*part/pieces,end=v[3]*Math.PI/180+sweep*(part+1)/pieces;lines.Add(Obj("id",id(part==0?e.e.Key+1:0),"start",Pt(v[0]+v[2]*Math.Cos(start),v[1]+v[2]*Math.Sin(start)),"end",Pt(v[0]+v[2]*Math.Cos(end),v[1]+v[2]*Math.Sin(end)),"type","interior","bowAngle",-sweep/pieces*180/Math.PI,"color",Color(EntityColor(e.e,Version))));}
    }else {
     int symbolId=id(e.e.Key+1);e.e.Key=symbolId-1;string name="native-apex/item-"+symbolId;List<O> geometry;double x=0,y=0,scale=1,rotation=0;
     var definition=NativeDefinition(name,e.e,page,out x,out y,out rotation,out scale);geometry=Items(definition,"geometry").ToList();
     custom[name]=definition;symbols.Add(Obj("id",symbolId,"symbolType",name,"x",x,"y",y,"scale",scale,"rotation",rotation));
    }}
    for(int ti=0;ti<page.Text.Count;ti++){var bytes=page.Text[ti];if((Version>=9&&bytes[0]==2)||page.Areas.Any(a=>(a.NameIndex>0&&a.NameIndex==ti)||(a.SizeIndex>0&&a.SizeIndex==ti)))continue;using(var r=new R(bytes,Version)){int token=r.ReadByte();var label=ReadText(r,token);if(Str(label,"text").Length==0)continue;label["id"]=id(Convert.ToInt32(label["id"])+1);labels.Add(label);}}
    foreach(var parent in page.Areas)foreach(int child in parent.Subtracts){var polygon=polygons.FirstOrDefault(poly=>Num(poly,"id")==child+1);if(polygon!=null){polygon["subtractTargetId"]=parent.Key+1;polygon["subtractMode"]="always";}}
    RestoreFaces(page,polygons,lines,ref next);
    RestoreDraftLines(page,polygons,lines,ref next);
    foreach(var draft in drafts)next=Math.Max(next,(int)Num(draft,"firstReplacementId"));
    pages.Add(Obj("name","Page "+(pages.Count+1),"lines",lines,"polygons",polygons,"labels",labels,"symbols",symbols,"reopenedAreas",drafts,"geoReferencePoints",GeoPoints(page,Version),"geoReferenceCoordinateSystem",GeoSystem(page,Version)));
   }
   var model=Obj("version",1,"name","Apex sketch","nextId",next,"pages",pages,"settings",RestoreOptions(Obj("customSymbols",custom)),"subjectInfo",Obj("fileNo",SubjectValue("FileNumber"),"propertyAddress",SubjectValue("PropertyAddress")));if(importedLayers!=null){model["layers"]=importedLayers;model["activeLayerId"]=importedActiveLayer;}return model;
  }
  O AreaTags(NativeArea area){var tags=Version>=35?Tags(area.Attributes,Version):new O();tags["apex.calculation"]="net";tags["apex.description"]=area.Description;tags["apex.averageWallHeight"]=area.Height.ToString("R",System.Globalization.CultureInfo.InvariantCulture);return tags;}
  string SubjectValue(string key){string value;return Subject.TryGetValue(key,out value)?value:"";}


  static int CharsetCode(byte charset){switch(charset){case 0:case 1:return 1252;case 177:return 1255;case 161:return 1253;case 162:return 1254;case 163:return 1258;case 186:return 1257;case 178:return 1256;case 238:return 1250;case 204:return 1251;default:throw new NotSupportedException("Unsupported native text character set "+charset);}}
  static int WinColor(string value){if(String.IsNullOrEmpty(value))return 0;int rgb;if(!Int32.TryParse(value.TrimStart('#'),System.Globalization.NumberStyles.HexNumber,System.Globalization.CultureInfo.InvariantCulture,out rgb))return 0;return (rgb&255)<<16|(rgb&65280)|((rgb>>16)&255);}
  static readonly short[] PaletteIds={0,1,2,3,4,5,6,7,9000,9001,9002,9003,9004,9005,9006,9007,9008,256};
  static readonly int[] PaletteRgb={0,255,65535,65280,16776960,16711680,16711935,16777215,8421504,128,32768,32896,8388608,8388736,8421376,12632256,0,8929280};
  static short PaletteIndex(int rgb){int index=Array.IndexOf(PaletteRgb,rgb);return index<0?(short)0:PaletteIds[index];}
  static int EntityColor(Ent entity,int version){if(version>=33)return BitConverter.ToInt32(entity.Prefix,40);int index=Array.IndexOf(PaletteIds,BitConverter.ToInt16(entity.Prefix,40));return index<0?0:PaletteRgb[index];}
  static byte[] TextRecord(O text,int version){return Pack(w=>{if(version>=9)w.Write((byte)1);byte charset=0;if(version>=35)w.Str(Str(text,"text"));else{
     byte[] bytes=null;foreach(byte candidate in new byte[]{0,204,238,161,162,163,177,178,186}){try{bytes=Encoding.GetEncoding(CharsetCode(candidate),EncoderFallback.ExceptionFallback,DecoderFallback.ExceptionFallback).GetBytes(Str(text,"text"));charset=candidate;break;}catch(EncoderFallbackException){}}
     if(bytes==null||bytes.Length>255)throw new NotSupportedException("Text cannot be represented in this older native format");w.Write((byte)6);w.Write((byte)bytes.Length);w.Write(bytes);
    }w.Write(WinColor(Str(text,"color")));w.Point(Num(text,"x"),Num(text,"y"));for(int i=0;i<4;i++){if(version>=21)w.Write(0);else w.Write((short)0);}int weight=Get(text,"bold",false).Equals(true)?700:400;if(version>=21)w.Write(weight);else w.Write((short)weight);w.Write(new[]{(byte)(Get(text,"italic",false).Equals(true)?1:0),(byte)0,(byte)0,charset});w.Write(0);w.Fixed(Str(text,"fontFamily","Arial"),32,false);w.Write(Num(text,"fontSize",12)/20);w.Write(0.0);w.Write((byte)0);w.Write(-Num(text,"rotation"));if(version>=30)w.Str("0");if(version>=4&&version<=8)w.Write((byte)0);if(version>=32)w.Write(checked((int)Num(text,"id")-1));},version);}
  static byte[] Layout(int version){return Pack(w=>{w.Empty();if(version>=2){w.Write(120.0);w.Write(0.0);w.Write(0.0);}if(version>=14)w.Write(new byte[12]);if(version>=15)w.Write(0);if(version>=25)w.Write(0);},version);}
  ApexFile Copy(){var f=(ApexFile)MemberwiseClone();f.Header=(byte[])Header.Clone();f.BodyPrefix=(byte[])BodyPrefix.Clone();f.Subject=new Dictionary<string,string>(Subject);f.Pages=Pages.ToList();return f;}

  static void Measure(NativePage page){foreach(var area in page.Areas){var vertices=page.Entities[area.Entity].Children;double signed=0,perimeter=0;for(int i=0;i<vertices.Count;i++){var a=vertices[i].Values;var b=vertices[(i+1)%vertices.Count].Values;double chord=Hypot(b[0]-a[0],b[1]-a[1]);signed+=(a[0]*b[1]-b[0]*a[1])/2;if(Math.Abs(a[5])<1e-12)perimeter+=chord;else{double angle=4*Math.Atan(a[5]),radius=chord*(1+a[5]*a[5])/(4*Math.Abs(a[5]));signed+=radius*radius*(angle-Math.Sin(angle))/2;perimeter+=radius*Math.Abs(angle);}}area.StoredArea=Math.Abs(signed)*area.Factor*(area.Sign==0?-1:1);area.Perimeter=perimeter;area.Wall=perimeter*area.Height;}}
  static double NativeNet(NativePage page,NativeArea area,HashSet<int> visiting){if(visiting.Count>256)throw new InvalidDataException("Native subtraction graph exceeds the depth limit");if(!visiting.Add(area.Key))throw new InvalidDataException("Cyclic native subtraction graph");double net=area.StoredArea;foreach(int id in area.Subtracts){var child=page.Areas.FirstOrDefault(a=>a.Key==id);if(child==null)throw new InvalidDataException("Missing native subtraction area");net-=NativeNet(page,child,visiting);}visiting.Remove(area.Key);return net;}
  public static ApexFile FromModel(O model,ApexFile source=null){
   if(source==null&&Num(Map(Get(Map(Get(model,"settings")),"apexDocument")),"units")==2){var metric=FromModel(model,New());metric.ScaleUnits(0.3048);metric.BodyPrefix[4]=2;return metric;}
   if(source!=null&&source.Measurement==2){var feet=Read(source.Write());feet.ScaleUnits(1/0.3048);feet.BodyPrefix[4]=5;var metric=FromModel(model,feet);metric.ScaleUnits(0.3048);metric.BodyPrefix[4]=2;return metric;}
   var f=(source??New()).Copy();var original=f.Pages.ToArray();var result=new List<NativePage>();int key=checked((int)Num(model,"nextId",1))-1;
   foreach(var page in Items(model,"pages")){


    var old=result.Count<original.Length?original[result.Count]:null;var p=new NativePage{Key=old==null?key++:old.Key,VisibleLayers=old==null?1:old.VisibleLayers,Layout=old==null?Layout(f.Version):old.Layout,Attributes=old==null?(f.Version>=37?new byte[4]:new byte[0]):old.Attributes};
    var symbolRecords=new O();p.Text.Add(TextRecord(Obj("text","","id",(key++)+1),f.Version));
    var lineMap=Items(page,"lines").ToDictionary(l=>checked((int)Num(l,"id")));var usedLines=new HashSet<int>();
    foreach(var poly in Items(page,"polygons")){
     var measured=Map(Get(poly,"apexBoundary"));var points=Items(measured.Count==0?poly:measured,"points").ToArray();if(points.Length<3)continue;int polyId=checked((int)Num(poly,"id"));var a=old==null?null:old.Areas.FirstOrDefault(v=>v.Key==polyId-1);if(a!=null){using(var r=new R(Pack(w=>WriteArea(w,a),f.Version),f.Version))a=ReadArea(r,r.ReadByte());}else a=new NativeArea();
     var tags=Map(Get(poly,"attributes"));a.Attributes=StoreTags(a.Attributes,tags,f.Version);a.Description=Str(tags,"apex.description",a.Description);a.Height=Num(tags,"apex.averageWallHeight",a.Height);
     a.Closed=1;a.Key=polyId-1;a.SecondKey=polyId-1;a.Code=Str(poly,"type","UND");a.Name=Str(poly,"name");double signedFactor=Num(poly,"apexFactor",Num(poly,"factorOverride",1));a.Factor=Math.Abs(signedFactor);a.Sign=(byte)(signedFactor<0?0:1);a.Entity=p.Entities.Count;a.NameIndex=0;a.SizeIndex=0;
     var boundary=Entity(13,key++,f.Version,0,0);boundary.Prefix=Common(f.Version,WinColor(Str(poly,"lineColor")));boundary.Flags=16385;var edges=Items(measured.Count==0?poly:measured,"edges").ToArray();bool face=measured.Count>0&&ModelJson.Serialize(Items(poly,"points").ToArray())!=ModelJson.Serialize(points);
     for(int i=0;i<points.Length;i++){double bow=0;int vertexId=key++;if(i<edges.Length){int lineId=checked((int)Num(edges[i],"lineId"));O line;if(!face&&lineMap.TryGetValue(lineId,out line)){usedLines.Add(lineId);vertexId=lineId-1;bow=Num(line,"bowAngle");var start=Map(Get(line,"start"));if(Math.Abs(Num(start,"x")-Num(points[i],"x"))>1e-7||Math.Abs(Num(start,"y")-Num(points[i],"y"))>1e-7)bow=-bow;}}boundary.Children.Add(Entity(14,vertexId,f.Version,Num(points[i],"x"),Num(points[i],"y"),0,0,0,Math.Tan(-bow*Math.PI/720)));}
     p.Entities.Add(boundary);p.Areas.Add(a);
    }
    foreach(var pair in lineMap){if(usedLines.Contains(pair.Key))continue;var start=Map(Get(pair.Value,"start"));var end=Map(Get(pair.Value,"end"));Ent entity;double bow=Num(pair.Value,"bowAngle");if(Math.Abs(bow)>1e-9){
     double bulge=Math.Tan(-bow*Math.PI/720),dx=Num(end,"x")-Num(start,"x"),dy=Num(end,"y")-Num(start,"y"),cx=(Num(start,"x")+Num(end,"x"))/2-dy*(1-bulge*bulge)/(4*bulge),cy=(Num(start,"y")+Num(end,"y"))/2+dx*(1-bulge*bulge)/(4*bulge);double first=Math.Atan2(Num(start,"y")-cy,Num(start,"x")-cx),last=Math.Atan2(Num(end,"y")-cy,Num(end,"x")-cx);if(bulge<0){double swap=first;first=last;last=swap;}entity=Entity(4,pair.Key-1,f.Version,cx,cy,Hypot(Num(start,"x")-cx,Num(start,"y")-cy),first*180/Math.PI,last*180/Math.PI);
    }else entity=Entity(2,pair.Key-1,f.Version,Num(start,"x"),Num(start,"y"),Num(end,"x"),Num(end,"y"));entity.Prefix=Common(f.Version,WinColor(Str(pair.Value,"color")));p.Entities.Add(entity);}
    foreach(var symbol in Items(page,"symbols")){
     string type=Str(symbol,"symbolType",Str(symbol,"type"));var definitions=Map(Get(model,"apexSymbols",Get(Map(Get(model,"settings")),"customSymbols")));var definition=Map(Get(definitions,type));
     if(!Items(definition,"geometry").Any())throw new InvalidDataException("Missing native symbol definition "+type);
     string name="SR"+Num(symbol,"id").ToString(System.Globalization.CultureInfo.InvariantCulture);var payload=Map(Get(definition,"nativeApex"));var block=payload.Count>0&&Str(payload,"geometryHash")==GeometryHash(Get(definition,"geometry"))?f.EditableNativeBlock(definition,payload,name,p,f.Version,ref key):new Block{Name=name,Layer="MSA",Entities=SymbolEntities(definition,f.Version,ref key),Tail=new byte[f.Version>=38?21:f.Version>=32?4:0]};p.Blocks.Add(block);if(payload.Count>0&&Str(payload,"geometryHash")==GeometryHash(Get(definition,"geometry")))symbolRecords[name]=Obj("blockHash",BlockHash(block),"geometry",Get(definition,"geometry"),"payload",payload);
     double scale=Num(symbol,"scale",1);var insert=Entity(23,checked((int)Num(symbol,"id")-1),f.Version,Num(symbol,"x"),Num(symbol,"y"),scale,scale,-Num(symbol,"rotation"));insert.Name=name;p.Entities.Add(insert);
    }
    Measure(p);
    var liveAreas=Items(page,"polygons").Where(poly=>Items(poly,"points").Count()>=3).ToArray();
    if(liveAreas.Any(poly=>Get(poly,"apexSubtractedFromId")!=null||Get(poly,"apexNet")!=null||Str(Map(Get(poly,"attributes")),"apex.calculation")=="net")){
        foreach(var area in p.Areas)area.Subtracts.Clear();
        foreach(var poly in liveAreas){object target=Get(poly,"apexSubtractedFromId",Str(poly,"subtractMode")=="always"?Get(poly,"subtractTargetId"):null);if(target!=null){int parent=checked(Convert.ToInt32(target)-1),child=checked((int)Num(poly,"id")-1);var container=p.Areas.FirstOrDefault(a=>a.Key==parent);if(container==null)throw new InvalidDataException("Missing native subtraction container");if(container.Sign==0)throw new NotSupportedException("Apex native format cannot represent a negative area that contains subtractions");container.Subtracts.Add(child);container.Sign=2;}}
        var nativeValues=new Dictionary<int,double>();foreach(var poly in liveAreas){var area=p.Areas.First(a=>a.Key==checked((int)Num(poly,"id")-1));double expected=Num(poly,"apexNet",area.StoredArea),native=NativeNet(p,area,new HashSet<int>());if(Get(poly,"apexNet")!=null&&Math.Abs(native-expected)>1e-6*Math.Max(1,Math.Abs(expected)))throw new NotSupportedException("This face measurement or factored/nested subtraction cannot yet be saved to native Apex without changing its measured area. The original file has not been overwritten; preserve this document as SKX.");nativeValues[area.Key]=native;}foreach(var area in p.Areas)area.StoredArea=nativeValues[area.Key];
    }
    var modelAreas=Items(page,"polygons").Where(poly=>Items(poly,"points").Count()>=3).ToArray();
    for(int ai=0;ai<p.Areas.Count;ai++){
     var area=p.Areas[ai];var poly=modelAreas[ai];var options=Map(Get(poly,"autoPost"));var vertices=p.Entities[area.Entity].Children;double cx=vertices.Average(v=>v.Values[0]),cy=vertices.Average(v=>v.Values[1]);var label=Map(Get(poly,"labelPosition"));if(label.Count!=0){cx=Num(label,"x");cy=Num(label,"y");}
     if(Object.Equals(Get(options,"name",true),true)){area.NameIndex=p.Text.Count;p.Text.Add(TextRecord(Obj("id",(key++)+1,"text",area.Name,"x",cx,"y",cy,"fontSize",14,"rotation",Num(poly,"labelRotation")),f.Version));}
     if(Object.Equals(Get(options,"calcs",true),true)){area.SizeIndex=p.Text.Count;var position=Map(Get(poly,"areaSizePosition"));p.Text.Add(TextRecord(Obj("id",(key++)+1,"text",area.StoredArea.ToString("0.##",System.Globalization.CultureInfo.InvariantCulture)+" sf","x",position.Count==0?cx:Num(position,"x"),"y",position.Count==0?cy-1:Num(position,"y"),"fontSize",12,"rotation",Num(poly,"areaSizeRotation",Num(poly,"labelRotation"))),f.Version));}
     if(Object.Equals(Get(options,"dims",true),true))for(int vi=0;vi<vertices.Count;vi++){
      var start=vertices[vi].Values;var end=vertices[(vi+1)%vertices.Count].Values;double dx=end[0]-start[0],dy=end[1]-start[1],chord=Hypot(dx,dy);if(chord<1e-12)continue;double length=chord;if(Math.Abs(start[5])>1e-12)length=chord*(1+start[5]*start[5])/(4*Math.Abs(start[5]))*Math.Abs(4*Math.Atan(start[5]));
      vertices[(vi+1)%vertices.Count].Label=checked((ushort)(p.Text.Count+1));
      var record=TextRecord(Obj("id",(key++)+1,"text",length.ToString("0.##",System.Globalization.CultureInfo.InvariantCulture)+"'","x",(start[0]+end[0])/2+dy/chord*0.5,"y",(start[1]+end[1])/2-dx/chord*0.5,"fontSize",12,"rotation",-Math.Atan2(dy,dx)*180/Math.PI),f.Version);
      if(f.Version>=9){record[0]=2;record=record.Concat(new byte[]{0}).ToArray();}p.Text.Add(record);
     }
    }
    foreach(var text in Items(page,"labels"))p.Text.Add(TextRecord(text,f.Version));p.Attributes=StoreGeo(p,page,f.Version);f.AddDrafts(page,p,model,ref key);StoreSymbolRecords(p,symbolRecords,f.Version,ref key);foreach(var block in p.Blocks)if(f.Version>=32)BitConverter.GetBytes(key++).CopyTo(block.Tail,0);p.AreaKey=key++;p.TextKey=key++;result.Add(p);
   }
   f.Pages=result;if(f.KeyOffset>=0)BitConverter.GetBytes(key).CopyTo(f.BodyPrefix,f.KeyOffset);return f;
  }
 }
}







