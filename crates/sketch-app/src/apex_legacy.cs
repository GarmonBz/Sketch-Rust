// Older desktop launch and XML/COM host contracts, executed inside sketch-app.exe.
using System;using System.IO;using System.Linq;using System.Xml;using System.Reflection;using System.Drawing;using System.Drawing.Imaging;using System.Collections.Generic;using System.Windows.Forms;
namespace SketchRust.Compat {
 public static class LegacyHost {
  public static XmlDocument ReadReport(string path){
   var settings=new XmlReaderSettings{DtdProcessing=DtdProcessing.Prohibit,XmlResolver=null,MaxCharactersInDocument=8*1024*1024};
   var doc=new XmlDocument{XmlResolver=null};using(var reader=XmlReader.Create(path,settings))doc.Load(reader);return doc;
  }
  public static Dictionary<string,string> Subject(XmlDocument report){
   var result=new Dictionary<string,string>();var root=report.DocumentElement;
   if(root==null)throw new InvalidDataException("Missing desktop report document");result["FileNumber"]=root.GetAttribute("FILENUM");
   foreach(var pair in new[]{new[]{"ADDRESS","PropertyAddress"},new[]{"CITY","City"},new[]{"COUNTY","County"},new[]{"STATE","State"},new[]{"ZIP","ZipCode"},new[]{"BORROWER","Borrower"},new[]{"LENDER","LenderClient"}}){var node=root.SelectSingleNode("PROPERTYINFO/"+pair[0]);if(node!=null)result[pair[1]]=node.InnerText;}
   return result;
  }
  public static XmlDocument Result(XmlDocument source,string sketch,ReturnData snapshot,string[] images){
   var doc=new XmlDocument();var report=doc.CreateElement("REPORT");doc.AppendChild(report);
   foreach(string name in new[]{"VERSION","FILENUM","DESCRIPTION","MAJORFORM"})report.SetAttribute(name,source.DocumentElement.GetAttribute(name));
   var data=doc.CreateElement("SKETCHDATA");data.SetAttribute("SKETCHFILE",sketch);var input=source.DocumentElement.SelectSingleNode("SKETCHDATA") as XmlElement;
   data.SetAttribute("PROPERTYTYPE",input==null?"NORMAL":input.GetAttribute("SKETCHMODETYPE"));report.AppendChild(data);
   var photos=doc.CreateElement("SKETCHPHOTOS");data.AppendChild(photos);
   for(int i=0;i<images.Length;i++){var photo=doc.CreateElement("SKETCHPHOTO");photo.SetAttribute("NUM",(i+1).ToString());var file=doc.CreateElement("FILENAME");file.InnerText=images[i];photo.AppendChild(file);var type=doc.CreateElement("FILETYPE");type.InnerText="EMF";photo.AppendChild(type);photos.AppendChild(photo);}
   var areas=doc.CreateElement("SKETCHAREAS");data.AppendChild(areas);
   foreach(var group in snapshot.pages.SelectMany(p=>p.areas).GroupBy(a=>a.code)){var area=doc.CreateElement("SKETCHAREA");area.SetAttribute("TYPE",group.Key);area.InnerText=group.Sum(a=>a.netArea).ToString("R",System.Globalization.CultureInfo.InvariantCulture);areas.AppendChild(area);}return doc;
  }
  static object Get(Type type,object api,string name){return type.InvokeMember(name,BindingFlags.GetProperty,null,api,null);}
  static void Image(string source,string path,float dpi){using(var input=System.Drawing.Image.FromFile(source))using(var image=new Bitmap(input))using(var bitmap=new Bitmap(1,1))using(var dc=Graphics.FromImage(bitmap)){var handle=dc.GetHdc();try{if(dpi>0)image.SetResolution(dpi,dpi);var frame=new RectangleF(0,0,image.Width*25.4f/image.HorizontalResolution,image.Height*25.4f/image.VerticalResolution);using(var meta=new Metafile(path,handle,frame,MetafileFrameUnit.Millimeter,EmfType.EmfPlusDual))using(var graphics=Graphics.FromImage(meta)){graphics.PageUnit=GraphicsUnit.Millimeter;graphics.Clear(Color.White);graphics.DrawImage(image,frame,new RectangleF(0,0,image.Width,image.Height),GraphicsUnit.Pixel);}}finally{dc.ReleaseHdc(handle);}}}
  public static void Run(Dictionary<string,string> boot){
   Session.Atomic(Path.Combine(Session.Root,"legacy-runtime.json"),Session.Json.Serialize(new{processId=System.Diagnostics.Process.GetCurrentProcess().Id,apexTransportLoaded=AppDomain.CurrentDomain.GetAssemblies().Any(a=>a.GetName().Name=="Apex.Integration.DotNet")}));
   string mode=boot["mode"],path=boot.ContainsKey("path")?boot["path"]:"";
   Type apiType=null;object api=null;XmlDocument report=null;string resultPath=null,folder=null;
   if(mode!="file"){
    string prog=mode.StartsWith("-WINTOTAL",StringComparison.OrdinalIgnoreCase)?"WTAPI.Sketch":"alamodeTOTALAPI.Sketch";
    apiType=Type.GetTypeFromProgID(prog,true);api=Activator.CreateInstance(apiType);
    if(!Convert.ToBoolean(Get(apiType,api,"LicenseValid")))throw new InvalidOperationException("The desktop host did not authorize this sketch integration session");
    folder=Convert.ToString(Get(apiType,api,"DataDir"));report=ReadReport(Convert.ToString(Get(apiType,api,"SourceXML")));resultPath=Convert.ToString(Get(apiType,api,"ResultXML"));
    var data=report.DocumentElement.SelectSingleNode("SKETCHDATA") as XmlElement;string filename=data==null?"":data.GetAttribute("SKETCHFILE");if(String.IsNullOrEmpty(filename))filename="Sketch.ax6";path=Path.Combine(folder,filename);
   }
   path=Path.GetFullPath(path);ApexFile native=File.Exists(path)?ApexFile.Read(File.ReadAllBytes(path)):null;
   var subject=report==null?(native==null?new Dictionary<string,string>():native.Subject):Subject(report);
   string document=null;if(native!=null){document=Path.Combine(Session.Root,"native-open.json");Session.Atomic(document,Session.Json.Serialize(native.ToModel()));}
   Session.Atomic(Path.Combine(Session.Root,"context.json"),Session.Json.Serialize(new{version=1,hostPath=path,fileMode=mode=="file",document=document,subject=subject,pageIndex=0,processId=System.Diagnostics.Process.GetCurrentProcess().Id}));
   File.WriteAllText(Path.Combine(Session.Root,"server-ready"),"");
   using(var timer=new System.Windows.Forms.Timer()){timer.Interval=100;bool ending=false;timer.Tick+=delegate{
    if(ending)return;
    if(File.Exists(Path.Combine(Session.Root,"cancel"))){ending=true;if(api!=null)apiType.InvokeMember("PostExit",BindingFlags.InvokeMethod,null,api,new object[]{false});Application.ExitThread();return;}
    bool closing=File.Exists(Path.Combine(Session.Root,"return-ready"));string ready=Path.Combine(Session.Root,closing?"return-ready":"save-ready");if(!File.Exists(ready))return;
    try{
     var model=Session.Json.Deserialize<Dictionary<string,object>>(File.ReadAllText(Path.Combine(Session.Root,File.Exists(Path.Combine(Session.Root,"return-model.json"))?"return-model.json":"model.json")));
     var output=ApexFile.FromModel(model,native);foreach(var pair in subject)output.Subject[pair.Key]=pair.Value;
     var bytes=output.Write();ApexFile.Read(bytes);Session.AtomicBytes(path,bytes);native=output;
     if(api!=null){var snapshot=Session.Json.Deserialize<ReturnData>(File.ReadAllText(Path.Combine(Session.Root,"result.json")));snapshot.ToNativeUnits(native!=null&&native.Measurement==2);var images=new string[snapshot.pages.Length];
      for(int i=0;i<images.Length;i++){images[i]=Path.Combine(folder,"SketchRust-"+(i+1)+".emf");Image(snapshot.pages[i].image,images[i],snapshot.pages[i].imageDpi);}
      var xml=Result(report,Path.GetFileName(path),snapshot,images);Session.Atomic(resultPath,xml.OuterXml);
      if(closing){int code=Convert.ToInt32(apiType.InvokeMember("PostExit",BindingFlags.InvokeMethod,null,api,new object[]{true}));if(code!=0)throw new InvalidOperationException("The desktop host rejected sketch return: "+code);}
     }
     ending=closing;File.Delete(ready);Session.Atomic(Path.Combine(Session.Root,closing?"return-receipt.json":"save-receipt.json"),"{\"ok\":true}");if(closing)Application.ExitThread();
    }catch(Exception error){File.Delete(ready);Session.Atomic(Path.Combine(Session.Root,closing?"return-receipt.json":"save-receipt.json"),Session.Json.Serialize(new{ok=false,error=error.Message}));}
   };timer.Start();Application.Run();}
   if(api!=null&&System.Runtime.InteropServices.Marshal.IsComObject(api))System.Runtime.InteropServices.Marshal.FinalReleaseComObject(api);
  }
 }
}
