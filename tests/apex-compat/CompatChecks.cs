// Protocol contract checks for our embedded component. No Apex program is started.
using System;
using System.IO;
using System.Collections;
using System.Drawing;
using System.Drawing.Imaging;
using System.Linq;
using Apex.Integration;
using Apex.Integration.Server;
using Apex.Integration.Client;
using SketchRust.Compat;
sealed class TestSink : ISketchSink {
 public int Saved, PageChanges;
 public void Clear(){} public void OnClosed(){} public bool OnClosing(){return false;}
 public void OnLoaded(){} public void OnModified(){} public void OnSaved(){Saved++;} public void OnPageChanged(){PageChanges++;}
 public void OnSymbolAdded(SymbolEventArgs e){} public void OnSymbolRemoved(SymbolEventArgs e){}
 public void OnAreaAdded(SketchItemEventArgs e){} public void OnAreaRemoved(SketchItemEventArgs e){}
 public void OnAreaChanged(SketchItemEventArgs e){} public void OnGeoReferencePointsPlaced(){}
}
static class CompatChecks {
 static int checks;
 static void Check(bool condition,string name){if(!condition)throw new Exception(name);checks++;}
 static void Reject(Action action,string name){try{action();}catch(NotSupportedException){checks++;return;}catch(ArgumentException){checks++;return;}catch(InvalidDataException){checks++;return;}catch(InvalidOperationException){checks++;return;}throw new Exception("Expected rejection: "+name);}
 static void Tags(ProxyBase owner){
  Check(owner.Count==0,"empty custom tags");owner.Add("report","original");owner.Add("report","updated");owner.Add("second","value");
  Check(owner.Count==2&&owner[0].Name=="report"&&owner["report"]=="updated","replace preserves order");
  Check(owner.Contains("second")&&owner.IndexOf("second")==1,"lookup");
  var copy=new ITag[3];owner.CopyTo(copy,1);Check(copy[1].Value=="updated"&&copy[2].Name=="second","CopyTo offset");
  Check(!owner.IsSynchronized&&owner.SyncRoot!=null,"collection metadata");
  Check(owner.Cast<ITag>().Count()==2,"enumeration");
  Check(owner.Remove("report")&&!owner.Remove("missing"),"remove");
  owner.Clear();Check(owner.Count==0&&!owner.Contains("second"),"clear");
 }
 static void Packet(string root,AreaData[] areas,params string[] images){
  File.WriteAllText(Path.Combine(root,"result.json"),Session.Json.Serialize(new ReturnData{pages=images.Select(p=>new PageData{image=p,areas=areas}).ToArray()}));
 }
 public static int Main(string[] args){try{
  string root=Path.GetFullPath(args[0]);Directory.CreateDirectory(root);Session.Root=root;
  string genuine=Path.Combine(root,"existing.ax7");byte[] original={1,2,3,4,5};File.WriteAllBytes(genuine,original);
  Reject(()=>new Document(new TestSink(),genuine),"existing native file");Check(File.ReadAllBytes(genuine).SequenceEqual(original),"original native file preserved");
  var sink=new TestSink();var doc=new Document(sink,Path.Combine(root,"new.ax7"));
  doc.Subject.FileNumber="HOST-42";doc.Subject.PropertyAddress="Test subject";
  Check(doc.Subject.FileNumber=="HOST-42"&&File.ReadAllText(Path.Combine(root,"context.json")).Contains("HOST-42"),"subject context");
  Tags(doc.Subject);Check(doc.Subject.FileNumber=="HOST-42","custom tags independent of standard subject");
  Reject(()=>doc.Save(),"no false native save success");Reject(()=>doc.SaveAsBase64String(),"no SKX disguised as native Base64");
  Reject(()=>{doc.PageIndex=-1;},"negative page");Reject(()=>{var p=doc[1];},"missing page");
  string first=Path.Combine(root,"first.png"),second=Path.Combine(root,"second.png");
  using(var image=new Bitmap(80,40)){using(var g=Graphics.FromImage(image))g.Clear(Color.Red);image.Save(first,ImageFormat.Png);}
  using(var image=new Bitmap(30,60)){using(var g=Graphics.FromImage(image))g.Clear(Color.Blue);image.Save(second,ImageFormat.Png);}
  var area=new AreaData{id=7,code="GLA1",name="Deduction",baseArea=200,netArea=-100,factor=-0.5,perimeter=60};
  File.WriteAllBytes(Path.Combine(root,"document.skx"),new byte[]{80,75,3,4}); // Editable bytes are validated by the separate Rust model tests.
  Packet(root,new[]{area},first,second);doc.AcceptReturn();
  Check(sink.Saved==1&&doc.Count==2&&doc[0][0].NetArea==-100,"signed multi-page return");
  Check(doc[0][0].Multiplier==-0.5&&doc[0][0].Sign==-1,"signed multiplier and sign");
  Check(Object.ReferenceEquals(doc[0],doc[0])&&Object.ReferenceEquals(doc[0][0],doc[0][0]),"stable proxy identity");Tags(doc[0][0]);
  doc.PageIndex=1;Check(sink.PageChanges==1,"page-changed callback");doc.PageIndex=1;Check(sink.PageChanges==1,"unchanged page has no callback");
  byte[] encoded=Convert.FromBase64String(doc.ExportPageAsBase64("PAGE:0"));using(var stream=new MemoryStream(encoded))using(var image=Image.FromStream(stream))Check(image.Width==30&&image.RawFormat.Guid==ImageFormat.Jpeg.Guid,"native Base64 current-page JPEG contract");
  using(var image=doc[1].ToSketchImage().ToImage())Check(image.Height==60,"ToImage contract");
  foreach(string ext in new[]{"png","jpg","bmp","gif","tiff","emf"}){
   string path=Path.Combine(root,"export."+ext);doc.ExportPageImages(path,ext=="emf"?"FORM:INTEGRATION PAGE:1":"PAGE:1");
   using(var image=Image.FromFile(path)){
    if(ext=="emf") {
     using(var source=Image.FromFile(second))Check(Math.Abs(image.Width/image.HorizontalResolution-source.Width/source.HorizontalResolution)<0.02&&Math.Abs(image.Height/image.VerticalResolution-source.Height/source.VerticalResolution)<0.02,"physical EMF page size");
    } else Check(image.Width==30&&image.Height==60,"image dimensions "+ext);
    if(ext!="emf")Check(image.RawFormat.Guid==ImageOptions.Format(path).Guid,"actual encoding "+ext);
   }
  }
  Reject(()=>doc.ExportPageImages(Path.Combine(root,"unknown.raw"),""),"unsupported image encoding");
  string framed=Path.Combine(root,"framed.png");
  using(var bitmap=new Bitmap(900,1376)){
   bitmap.SetResolution(300,300);using(var graphics=Graphics.FromImage(bitmap)){
    graphics.Clear(Color.White);graphics.FillRectangle(Brushes.Red,0,0,900,10);
    graphics.FillRectangle(Brushes.Red,0,1366,900,10);graphics.FillRectangle(Brushes.Red,0,0,10,1376);
    graphics.FillRectangle(Brushes.Red,890,0,10,1376);
   }bitmap.Save(framed);
  }
  var framedPage=new Page(new PageData{image=framed,imageDpi=300,areas=new AreaData[0]});
  foreach(string type in new[]{"GDI","GDIPLUS","DUAL"}){
   string output=Path.Combine(root,"framed-"+type+".emf");framedPage.SaveImage(output,"FORM:INTEGRATION EMFTYPE:"+type);
   using(var metafile=Image.FromFile(output))using(var bitmap=new Bitmap(metafile,new Size(640,980))){
    using(var graphics=Graphics.FromImage(bitmap)){graphics.Clear(Color.White);graphics.DrawImage(metafile,0,0,bitmap.Width,bitmap.Height);}
    foreach(var point in new[]{new Point(2,490),new Point(637,490),new Point(320,2),new Point(320,977)}){
     var color=bitmap.GetPixel(point.X,point.Y);Check(color.R>180&&color.G<100&&color.B<100,"full metafile frame "+type+" at "+point);
    }
   }
  }
  Reject(()=>doc.ExportPageAsBase64("PAGE:99"),"invalid image page");Reject(()=>doc.ExportPageAsBase64("PAGE:bad"),"malformed page option");
  area.factor=Double.PositiveInfinity;Packet(root,new[]{area},first);Reject(()=>doc.AcceptReturn(),"nonfinite factor");
  Check(doc.Count==2&&sink.Saved==1,"bad return preserves previous snapshot");area.factor=1;
  Packet(root,new[]{area,area},first);Reject(()=>doc.AcceptReturn(),"duplicate area ID");
  var sourceXml=new System.Xml.XmlDocument();sourceXml.LoadXml("<REPORT VERSION='1' FILENUM='LEGACY-TEST' DESCRIPTION='Fixture' MAJORFORM='1004'><PROPERTYINFO><ADDRESS>Test &amp; Verify</ADDRESS><CITY>Test City</CITY><STATE>IL</STATE><ZIP>60000</ZIP></PROPERTYINFO><SKETCHDATA SKETCHFILE='report.ax6' SKETCHMODETYPE='NORMAL'><SKETCHAREAS/></SKETCHDATA></REPORT>");
  var hostSubject=LegacyHost.Subject(sourceXml);Check(hostSubject["FileNumber"]=="LEGACY-TEST"&&hostSubject["PropertyAddress"]=="Test & Verify","legacy input subject");
  var legacyResult=LegacyHost.Result(sourceXml,"report.ax6",new ReturnData{pages=new[]{new PageData{areas=new[]{new AreaData{code="GLA1",netArea=200},new AreaData{code="GLA1",netArea=-50}}}}},new[]{"page.emf"});
  Check(legacyResult.DocumentElement.GetAttribute("FILENUM")=="LEGACY-TEST","legacy report identity");
  Check(legacyResult.SelectSingleNode("REPORT/SKETCHDATA/SKETCHAREAS/SKETCHAREA").InnerText=="150","legacy signed area aggregation");
  Check(legacyResult.SelectSingleNode("REPORT/SKETCHDATA/SKETCHPHOTOS/SKETCHPHOTO/FILETYPE").InnerText=="EMF","legacy page image contract");
  var geographic=new Page(new PageData{areas=new AreaData[0],geoReferencePoints=new[]{new GeoData{x=3.5,y=-2,coordinates="41 -87"}}});
  Check(geographic.GeoReferencePointCount==1&&geographic.GetGeoReferencePoint(0).SketchY==-2,"georeference axes");geographic.GetGeoReferencePoint(0).GeoCoordinates="42 -88";
  Check(geographic.GetGeoReferencePoint(0).GeoCoordinates=="42 -88","georeference assignment");
  Console.WriteLine("PASS: "+checks+" embedded component contract checks; no external sketch process started.");return 0;
 }catch(Exception error){Console.Error.WriteLine(error);return 1;}}
}
