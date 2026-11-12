// Reference-only conformance test. This is never linked into the sketch application.
using System;using System.IO;using System.Reflection;
class ReferenceFileChecks {
 static Assembly assembly;static Type Type(string n){return assembly.GetType("Apex.Sketch.Axf."+n,true);}
 static object New(string n){return Activator.CreateInstance(Type(n));}
 static object Field(object o,string n){return o.GetType().GetField(n).GetValue(o);}
 static object Property(object o,string n){return o.GetType().GetProperty(n).GetValue(o,null);}
 static void Set(object o,string n,object v){o.GetType().GetField(n).SetValue(o,v);}
 static void Add(object list,object o){list.GetType().GetMethod("Add").Invoke(list,new[]{o});}
 static int Main(string[] args){try{
  string folder=Path.GetDirectoryName(args[0]);AppDomain.CurrentDomain.AssemblyResolve+=(s,e)=>{string name=new AssemblyName(e.Name).Name+".dll";foreach(var dir in new[]{folder,Path.Combine(folder,"Library")}){string p=Path.Combine(dir,name);if(File.Exists(p))return Assembly.LoadFrom(p);}return null;};assembly=Assembly.LoadFrom(args[0]);
  foreach(string p in Directory.GetFiles(args[1],"*.ax7")){var sketch=New("AxfSketch");using(var stream=File.OpenRead(p))Type("AxfSketch").GetMethod("Load",new[]{typeof(Stream)}).Invoke(sketch,new[]{stream});Console.WriteLine("Reference reader accepts "+Path.GetFileName(p));}
  foreach(var expected in new[]{new object[]{"nested-affine-point.ax7",new[]{-100.0}},new object[]{"all-library-symbols.ax7",new[]{-100.0}},new object[]{"unfinished-v38.ax7",new[]{0.0}},new object[]{"metric.ax7",new[]{200.0}},new object[]{"face-measured.ax7",new[]{144.0}},new object[]{"factored-nested.ax7",new[]{107.5,-7.5,20.0}}}){
   string path=Path.Combine(args[1],(string)expected[0]);if(!File.Exists(path))continue;var document=Activator.CreateInstance(assembly.GetType("Apex.Sketch.SketchDocument",true));using(var stream=File.OpenRead(path))document.GetType().GetMethod("Load",new[]{typeof(Stream)}).Invoke(document,new[]{stream});var pages=(System.Collections.IEnumerable)Property(document,"Pages");var iterator=pages.GetEnumerator();iterator.MoveNext();var areas=(System.Collections.IEnumerable)Property(iterator.Current,"Areas");var actual=new System.Collections.Generic.List<double>();foreach(var area in areas)actual.Add(Convert.ToDouble(Property(area,"NetArea")));var values=(double[])expected[1];if(actual.Count!=values.Length)throw new Exception("Reference area count: "+path);for(int i=0;i<values.Length;i++)if(Math.Abs(actual[i]-values[i])>1e-6)throw new Exception("Reference native calculation mismatch: "+path+" = "+actual[i]);Console.WriteLine("Reference drawing engine confirms measurements for "+Path.GetFileName(path));
  }
  for(byte version=7;version<=38;version++){
   var sketch=New("AxfSketch");var header=Field(sketch,"Header");header.GetType().GetProperty("FileVersion").SetValue(header,version,null);
   var page=New("AxfPage");var boundary=New("AxfPolylineEntity");Set(boundary,"PolylineFlag",(short)16385);boundary.GetType().GetProperty("KeyCode").SetValue(boundary,30,null);
   foreach(var xy in new[]{new[]{0.0,0.0},new[]{20.0,0.0},new[]{20.0,-10.0},new[]{0.0,-10.0}}){var v=Activator.CreateInstance(Type("AxfVertexEntity"),new object[]{xy[0],xy[1]});Add(Field(boundary,"Vertices"),v);}
   Add(Field(page,"Entities"),boundary);var area=New("AxfArea");Set(area,"AreaType","GLA1");Set(area,"AreaName","Reference first floor");Set(area,"AreaDescription","Preserve description");Set(area,"EntityIndex",0);Set(area,"Closed",(byte)1);area.GetType().GetProperty("KeyCode").SetValue(area,20,null);Add(Field(page,"Areas"),area);
   Add(Property(sketch,"Pages"),page);string output=Path.Combine(args[2],"reference-rectangle-v"+version+".ax7");using(var stream=File.Create(output))Type("AxfSketch").GetMethod("Save",new[]{typeof(Stream)}).Invoke(sketch,new[]{stream});
  }
  Console.WriteLine("PASS: reference reader accepted independently written files; populated reference fixtures created.");return 0;
 }catch(Exception e){Console.Error.WriteLine(e);return 1;}}
}
