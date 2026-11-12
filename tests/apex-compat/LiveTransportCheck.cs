using System;
using System.IO;
using System.Diagnostics;
using System.Drawing;
using System.Reflection;
using System.Linq;
using System.Threading;
using System.Web.Script.Serialization;
using Apex.Integration;
using Apex.Integration.Server;
using Apex.Integration.Client;
using SketchRust.Compat;
public class LiveSink:MarshalByRefObject,ISketchSink,ISketchClientSink {
 public static int Saved,Loaded,Removed;public override object InitializeLifetimeService(){return null;}
 public void Clear(){}public void OnClosed(){}public bool OnClosing(){return false;}public void OnLoaded(){Loaded++;}public void OnModified(){}public void OnSaved(){Saved++;}public void OnPageChanged(){}
 public void OnSymbolAdded(SymbolEventArgs e){}public void OnSymbolRemoved(SymbolEventArgs e){}public void OnAreaAdded(SketchItemEventArgs e){}public void OnAreaRemoved(SketchItemEventArgs e){Removed++;}public void OnAreaChanged(SketchItemEventArgs e){}public void OnGeoReferencePointsPlaced(){}
 public void OnApplicationClosed(){}public bool OnApplicationClosing(){return false;}
}
public static class LiveCheck {
 static Type Find(Assembly a,string name){return a.GetTypes().Single(t=>t.Name==name);}
 static void WaitFor(string file){var timer=Stopwatch.StartNew();while(!File.Exists(file)){if(timer.ElapsedMilliseconds>20000)throw new Exception("Timed out: "+file);Thread.Sleep(50);}}
 public static int Main(string[] args){
  Process child=null;IDisposable activeClient=null;System.Threading.Tasks.Task<string> output=null,error=null;
  try{
   string root=args[1];Directory.CreateDirectory(root);
   var model=ApexFile.Read(File.ReadAllBytes(args[2])).ToModel();foreach(var page in ApexFile.Items(model,"pages"))foreach(var area in ApexFile.Items(page,"polygons"))area["factorOverride"]=1.0;
   foreach(var page in ApexFile.Items(model,"pages"))page["geoReferencePoints"]=new[]{ApexFile.Obj("x",1,"y",2,"coordinates",""),ApexFile.Obj("x",19,"y",9,"coordinates","")};
   string fixture=Path.Combine(root,"fixture.json");File.WriteAllText(fixture,new JavaScriptSerializer{MaxJsonLength=64*1024*1024}.Serialize(model));
   string guid=Guid.NewGuid().ToString();string config=Path.Combine(root,"input.xml");File.WriteAllText(config,"<product name=\"ApexSketch\" xmlns=\"urn:xmlns:apex:integration:configuration\"><server>"+guid+"</server></product>");
   var start=new ProcessStartInfo(args[0],"--check-apex-live \""+config+"\" \""+root+"\" \""+fixture+"\"");start.UseShellExecute=false;start.CreateNoWindow=true;start.RedirectStandardError=true;start.RedirectStandardOutput=true;
   var process=child=Process.Start(start);output=process.StandardOutput.ReadToEndAsync();error=process.StandardError.ReadToEndAsync();{
    if(!process.WaitForInputIdle(15000))throw new Exception("Provider did not become input-idle for the host startup contract");
    WaitFor(Path.Combine(root,"server-ready"));
    var a=typeof(SketchServer).Assembly;var resources=Find(a,"Resources");
    Func<string,string> text=n=>(string)resources.GetProperty(n,BindingFlags.Public|BindingFlags.NonPublic|BindingFlags.Static).GetValue(null,null);
    string uri=string.Format(text("Apex_Integration_Ipc_AddressFormat"),guid,text("Apex_Integration_Ipc_ServerObject"));
    var provider=Activator.CreateInstance(Find(a,"IpcChannelProvider"),new object[]{Guid.NewGuid().ToString()});
    var client=Activator.CreateInstance(Find(a,"RemotingClient"),new object[]{provider,uri,15000});activeClient=(IDisposable)client;
    var server=client.GetType().GetProperty("Server").GetValue(client,null);var accept=server.GetType().GetMethod("Accept");var sink=new LiveSink();
    var reg=Find(a,"RegisterClientMessage");var mode=reg.GetNestedType("Registration");accept.Invoke(server,new[]{Activator.CreateInstance(reg,new object[]{sink,Enum.Parse(mode,"Register")})});
    var msg=Activator.CreateInstance(Find(a,"NewSketchMessage"),new object[]{sink,Path.Combine(root,"test-report.ax7")});accept.Invoke(server,new[]{msg});
    var document=(ISketch)msg.GetType().GetProperty("Sketch").GetValue(msg,null);
    var proxy=(SketchProxy)document.GetType().GetField("proxy",BindingFlags.NonPublic|BindingFlags.Instance).GetValue(document);
    WaitFor(Path.Combine(root,"editor-ready"));
    if(document.Count!=1||document[0].Count!=1||Math.Abs(document[0][0].NetArea-200)>1e-6)throw new Exception("The live editor fixture was not returned");
    document.Subject.FileNumber="LIVE-API";document.Subject.PropertyAddress="Disposable native API fixture";
    document[0][0].Tags.Add("live-property","persist this");document.Subject.Tags.Add("subject-property","persist too");
    document.Save();string saved=Path.Combine(root,"test-report.ax7");var native=ApexFile.Read(File.ReadAllBytes(saved));if(native.Pages[0].Areas.Count!=1)throw new Exception("Native Save lost geometry");
    var savedArea=ApexFile.Items(ApexFile.Items(native.ToModel(),"pages").First(),"polygons").First();if(ApexFile.Str(ApexFile.Map(ApexFile.Get(savedArea,"attributes")),"live-property")!="persist this"||native.Subject["subject-property"]!="persist too")throw new Exception("Native tag persistence failed");
    string copy=Path.Combine(root,"saved-as.ax7");document.SaveAs(copy);if(!File.Exists(copy))throw new Exception("Save As failed");
    var base64=Convert.FromBase64String(proxy.SaveAsBase64String());if(ApexFile.Read(base64).Subject["FileNumber"]!="LIVE-API")throw new Exception("Native Base64 or subject failed");
    if(document[0].GeoReferencePoints.Count!=2||document[0].GeoReferencePoints[0].SketchY!=2)throw new Exception("Live georeference positions failed");
    document[0].GeoReferencePoints[0].GeoCoordinates="41.0 -87.0";document[0].GeoReferencePoints.CoordinateSystem="GEOGCS[\"WGS 84\"]";
    if(document[0].GeoReferencePoints[0].GeoCoordinates!="41.0 -87.0")throw new Exception("Live georeference setter failed");
    var breakdown=document[0][0].Breakdown();if(breakdown.Count==0||Math.Abs(breakdown[0].Area-200)>1e-6)throw new Exception("Area breakdown was empty or incorrect");
    int id=document[0][0].UniqueId;if(!document.ZoomToItem(id)||document.ZoomToItem(int.MaxValue))throw new Exception("Zoom identity failed");
    document[0][0].Delete();if(document[0].Count!=0)throw new Exception("Area.Delete did not update the editor");
    proxy.Load(copy,false,0);if(document[0].Count!=1||Math.Abs(document[0][0].NetArea-200)>1e-6)throw new Exception("Native Load failed");
    proxy.Load(Convert.ToBase64String(base64),true,0);if(document[0].Count!=1)throw new Exception("Native Base64 Load failed");
    proxy.ExportPageImages(Path.Combine(root,"live.emf"),"PAGE:0 FORM:INTEGRATION");
    proxy.ExportPageImages(Path.Combine(root,"letter.emf"),"FORM:STANDARD FORM:LETTER PAGE:0 EMFTYPE:GDI EMFDPI:150 PAGEINDEX");
    using(var image=System.Drawing.Image.FromFile(Path.Combine(root,"letter.emf"))){if(Math.Abs(image.Width/image.HorizontalResolution-8.5)>0.1)throw new Exception("Report form paper size failed");}
    int savedBefore=LiveSink.Saved;File.WriteAllText(Path.Combine(root,"ui-save"),"");WaitFor(Path.Combine(root,"save-receipt.json"));if(LiveSink.Saved<=savedBefore)throw new Exception("Native UI Save did not notify its host");
    string fixtureFolder=Path.GetDirectoryName(args[2]);
    proxy.Load(Path.Combine(fixtureFolder,"metric.ax7"),false,0);if(Math.Abs(document[0][0].NetArea-200)>1e-5||Math.Abs(document[0][0].Perimeter-60)>1e-5)throw new Exception("Metric native API units changed");document.SaveAs(Path.Combine(root,"metric-saved.ax7"));if(ApexFile.Read(File.ReadAllBytes(Path.Combine(root,"metric-saved.ax7"))).Measurement!=2)throw new Exception("Metric Save lost native unit");
    proxy.Load(Path.Combine(fixtureFolder,"factored-nested.ax7"),false,0);if(document[0].Count!=3||Math.Abs(document[0][0].NetArea-107.5)>1e-5||Math.Abs(document[0][1].NetArea+7.5)>1e-5)throw new Exception("Nested/factored native live calculation failed");document.SaveAs(Path.Combine(root,"graph-saved.ax7"));
    proxy.Load(Path.Combine(fixtureFolder,"nested-affine-point.ax7"),false,0);document.SaveAs(Path.Combine(root,"cad-saved.ax7"));if(!ApexFile.Read(File.ReadAllBytes(Path.Combine(root,"cad-saved.ax7"))).Pages[0].Blocks.SelectMany(b=>b.Entities).Any(e=>e.Type==1))throw new Exception("Rust symbol persistence lost native point metadata");
    proxy.Load(Path.Combine(fixtureFolder,"face-measured.ax7"),false,0);if(Math.Abs(document[0][0].NetArea-144)>1e-5)throw new Exception("Native face calculation was not restored");document.SaveAs(Path.Combine(root,"face-saved.ax7"));
    var second=Activator.CreateInstance(Find(a,"NewSketchMessage"),new object[]{sink,Path.Combine(root,"second-report.ax7")});accept.Invoke(server,new[]{second});document=(ISketch)second.GetType().GetProperty("Sketch").GetValue(second,null);if(document.Count!=1||document[0].Count!=0)throw new Exception("Repeated New did not replace the editor document");
    document.Close();if(!process.WaitForExit(30000))throw new Exception("Host close did not terminate its test session");
    Console.Write(output.GetAwaiter().GetResult());Console.Write(error.GetAwaiter().GetResult());
    if(process.ExitCode!=0||LiveSink.Saved<2||LiveSink.Loaded<2||LiveSink.Removed<1)throw new Exception("Live callbacks were incomplete");
    Console.WriteLine("PASS: genuine native Save/SaveAs/Base64/Load, live areas, tags, breakdown, zoom, Delete, EMF, callbacks and close.");((IDisposable)client).Dispose();
   }return 0;
  }catch(Exception e){Console.Error.WriteLine(e);if(child!=null){try{if(!child.HasExited){File.WriteAllText(Path.Combine(args[1],"host-closed"),"");if(!child.WaitForExit(3000))child.Kill();}child.WaitForExit(5000);if(output!=null&&output.IsCompleted)Console.Write(output.Result);if(error!=null&&error.IsCompleted)Console.Error.Write(error.Result);}catch{}}return 1;}
 finally{if(activeClient!=null){var cleanup=new Thread(delegate(){try{activeClient.Dispose();}catch{}});cleanup.IsBackground=true;cleanup.Start();cleanup.Join(2000);}if(child!=null){try{if(!child.HasExited)child.Kill();}catch{}child.Dispose();}}
 }
}
