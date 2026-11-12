// Host-supplied code tables. Loaded and parsed by the sketch EXE itself.
using System;using System.IO;using System.Linq;using System.Net;using System.Xml;using System.Text;using System.Globalization;using System.Collections.Generic;using System.Web.Script.Serialization;
using O=System.Collections.Generic.Dictionary<string,object>;
namespace SketchRust.Compat {
 public static class HostCodes {
  const int Limit=1024*1024;
  static double Number(XmlElement e,string name,double fallback){string s=e.GetAttribute(name);if(s.Length==0)return fallback;double value;if(!Double.TryParse(s,NumberStyles.Float,CultureInfo.InvariantCulture,out value)||Double.IsNaN(value)||Double.IsInfinity(value))throw new InvalidDataException("Invalid host code-table number: "+name);return value;}
  static string Color(double value){int c=checked((int)value);return "#"+((c&255)<<16|(c&65280)|((c>>16)&255)).ToString("x6");}
  public static string Parse(byte[] bytes){
   if(bytes.Length>Limit)throw new InvalidDataException("Host code table exceeds 1 MiB");var settings=new XmlReaderSettings{DtdProcessing=DtdProcessing.Prohibit,XmlResolver=null,MaxCharactersInDocument=Limit};var document=new XmlDocument{XmlResolver=null};using(var stream=new MemoryStream(bytes))using(var reader=XmlReader.Create(stream,settings))document.Load(reader);
   var table=document.SelectSingleNode("//*[local-name()='areaCodeTable']");if(table==null)throw new InvalidDataException("Missing host areaCodeTable element");var result=new List<O>();var used=new HashSet<string>(StringComparer.OrdinalIgnoreCase);
   foreach(XmlElement entry in table.SelectNodes(".//*[local-name()='areaCodeEntry']")){
    string code=entry.GetAttribute("code").Trim();if(code.Length==0)continue;if(code.Length>128||!used.Add(code))throw new InvalidDataException("Invalid or duplicate host area code");if(result.Count>=4096)throw new InvalidDataException("Too many host area codes");
    double factor=Number(entry,"multiplier",1);string positive=entry.GetAttribute("positive");bool sign=true;if(positive.Length!=0&&!Boolean.TryParse(positive,out sign))throw new InvalidDataException("Invalid host code-table sign");factor*=sign?1:-1;string calc=entry.GetAttribute("calculationType");bool living=calc=="LivingArea"||(calc.Length==0||calc=="NotDefined")&&code.StartsWith("GLA",StringComparison.OrdinalIgnoreCase);string fill=entry.GetAttribute("fill");
    result.Add(ApexFile.Obj("code",code,"name",entry.GetAttribute("areaName"),"description",entry.GetAttribute("areaDescription"),"group",living?"GLA":calc=="BuildingArea"||code.StartsWith("GBA",StringComparison.OrdinalIgnoreCase)?"GBA":"OTH","includeInGLA",living,"multiplier",Math.Abs(factor),"positive",factor>=0,"color",Color(Number(entry,"color",0)),"fillColor",Color(Number(entry,"fillColor",16777215)),"fillStyle",fill=="65535"||fill=="Transparent"||fill=="Empty"?"none":"solid","nativeProperties",ApexFile.Obj("wallHeight",Number(entry,"height",8),"smartLabelCode",entry.GetAttribute("smartLabelCode")),"fillOpacity",fill=="65535"||fill=="Transparent"||fill=="Empty"?0:10));
   }
   if(result.Count==0)throw new InvalidDataException("Host area code table is empty");return new JavaScriptSerializer{MaxJsonLength=4*Limit}.Serialize(result);
  }
  public static byte[] Fetch(string address){
   Uri uri;if(!Uri.TryCreate(address,UriKind.Absolute,out uri)||(!uri.IsFile&&uri.Scheme!="http"&&uri.Scheme!="https"))uri=new Uri(Path.GetFullPath(address));
   using(var output=new MemoryStream()){
    if(uri.IsFile){using(var input=File.OpenRead(uri.LocalPath))Copy(input,output);}
    else{var request=(HttpWebRequest)WebRequest.Create(uri);request.Timeout=10000;request.ReadWriteTimeout=10000;request.MaximumAutomaticRedirections=3;using(var response=request.GetResponse())using(var input=response.GetResponseStream())Copy(input,output);}
    return output.ToArray();
   }
  }
  static void Copy(Stream input,MemoryStream output){var buffer=new byte[8192];int count;while((count=input.Read(buffer,0,buffer.Length))>0){if(output.Length+count>Limit)throw new InvalidDataException("Host code table exceeds 1 MiB");output.Write(buffer,0,count);}}
 }
 public static partial class BootCodes {
  public static int Import(string argsFile){var config=new JavaScriptSerializer().Deserialize<Dictionary<string,string>>(File.ReadAllText(argsFile));File.WriteAllText(config["output"],HostCodes.Parse(HostCodes.Fetch(config["url"])));return 0;}
 }
}
