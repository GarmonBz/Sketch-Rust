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
public class Sink:MarshalByRefObject,ISketchSink,ISketchClientSink {
 public static ISketch Document; public static string Root; public static bool Saved,Closed;
 public override object InitializeLifetimeService(){return null;}
 public void Clear(){} public void OnClosed(){} public bool OnClosing(){return false;}
 public void OnLoaded(){}public void OnModified(){}public void OnPageChanged(){}
 public void OnSymbolAdded(SymbolEventArgs e){}public void OnSymbolRemoved(SymbolEventArgs e){}
 public void OnAreaAdded(SketchItemEventArgs e){}public void OnAreaRemoved(SketchItemEventArgs e){}public void OnAreaChanged(SketchItemEventArgs e){}public void OnGeoReferencePointsPlaced(){}
 public bool OnApplicationClosing(){return false;} public void OnApplicationClosed(){Closed=true;}
 public void OnSaved(){
   if(Document.Count!=2||Document[0].Count!=1||Document[0][0].NetArea!=200||Document[0][0].Code!="GLA1")throw new Exception("Returned geometry data differs");
   ((SketchProxy)Document.GetType().GetField("proxy",BindingFlags.NonPublic|BindingFlags.Instance).GetValue(Document)).ExportPageImages(Path.Combine(Root,"returned.emf"),"FORM:INTEGRATION PAGE:0");
   using(var image=Image.FromFile(Path.Combine(Root,"returned.emf"))) if(image.Width<1)throw new Exception("Invalid EMF");
   Saved=true;
 }
}
// Test client only. Never distributed with the application.
public static class Check {
 static Type Find(Assembly a,string name){return a.GetTypes().Single(t=>t.Name==name);}
 static void WaitFor(string file){var timer=Stopwatch.StartNew();while(!File.Exists(file)){if(timer.ElapsedMilliseconds>20000)throw new Exception("Timed out: "+file);Thread.Sleep(50);}}
 public static int Main(string[] args){
  try{
   string root=args[1];Directory.CreateDirectory(root);Sink.Root=root;
   string guid=Guid.NewGuid().ToString();string config=Path.Combine(root,"input.xml");File.WriteAllText(config,"<product name=\"ApexSketch\" xmlns=\"urn:xmlns:apex:integration:configuration\"><server>"+guid+"</server></product>");
   var start=new ProcessStartInfo(args[0],"--check-apex-server \""+config+"\" \""+root+"\"");start.UseShellExecute=false;start.CreateNoWindow=true;start.RedirectStandardError=true;start.RedirectStandardOutput=true;
   using(var process=Process.Start(start)){
    if(!process.WaitForInputIdle(15000))throw new Exception("Provider did not become input-idle for the host startup contract");
    WaitFor(Path.Combine(root,"server-ready"));
    var a=typeof(SketchServer).Assembly;var resources=Find(a,"Resources");
    Func<string,string> text=n=>(string)resources.GetProperty(n,BindingFlags.Public|BindingFlags.NonPublic|BindingFlags.Static).GetValue(null,null);
    string uri=string.Format(text("Apex_Integration_Ipc_AddressFormat"),guid,text("Apex_Integration_Ipc_ServerObject"));
    var provider=Activator.CreateInstance(Find(a,"IpcChannelProvider"),new object[]{Guid.NewGuid().ToString()});
    var client=Activator.CreateInstance(Find(a,"RemotingClient"),new object[]{provider,uri,15000});
    var server=client.GetType().GetProperty("Server").GetValue(client,null);var accept=server.GetType().GetMethod("Accept");var sink=new Sink();
    var reg=Find(a,"RegisterClientMessage");var mode=reg.GetNestedType("Registration");accept.Invoke(server,new[]{Activator.CreateInstance(reg,new object[]{sink,Enum.Parse(mode,"Register")})});
    var msg=Activator.CreateInstance(Find(a,"NewSketchMessage"),new object[]{sink,Path.Combine(root,"test-report.ax7")});accept.Invoke(server,new[]{msg});
    Sink.Document=(ISketch)msg.GetType().GetProperty("Sketch").GetValue(msg,null);
    Sink.Document.Subject.FileNumber="TEST-001";Sink.Document.Subject.PropertyAddress="Disposable protocol fixture";
    WaitFor(Path.Combine(root,"context.json"));if(!File.ReadAllText(Path.Combine(root,"context.json")).Contains("TEST-001"))throw new Exception("Subject data did not arrive");
    string png=Path.Combine(root,"page.png");using(var bmp=new Bitmap(600,400))using(var g=Graphics.FromImage(bmp)){g.Clear(Color.White);g.DrawRectangle(Pens.Black,30,30,500,300);bmp.Save(png,System.Drawing.Imaging.ImageFormat.Png);}
    // This checks byte transport; the Rust model suite validates real editable SKX.
    File.WriteAllBytes(Path.Combine(root,"document.skx"),new byte[]{80,75,3,4});
    var area=new {id=5,code="GLA1",name="First Floor",baseArea=200,netArea=200,factor=1,perimeter=60};var page=new {image=png,areas=new[]{area}};
    File.WriteAllText(Path.Combine(root,"result.json"),new JavaScriptSerializer().Serialize(new {pages=new[]{page,page}}));File.WriteAllText(Path.Combine(root,"return-ready"),"");
    if(!process.WaitForExit(30000))throw new Exception("Native process did not exit");
    string output=process.StandardOutput.ReadToEnd(),error=process.StandardError.ReadToEnd();Console.Write(output);Console.Write(error);
    if(process.ExitCode!=0||!Sink.Saved||!Sink.Closed)throw new Exception("Return callbacks failed");
    if(!File.Exists(Path.Combine(root,"test-report.ax7.sketch-rust.skx")))throw new Exception("Editable state was not saved");
    Console.WriteLine("PASS: embedded server; subject transfer; two pages; 200 sf; EMF export; save/close callbacks; state-byte persistence.");((IDisposable)client).Dispose();
   }return 0;
  }catch(Exception e){Console.Error.WriteLine(e);return 1;}
 }
}
