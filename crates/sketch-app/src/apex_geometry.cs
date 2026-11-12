// Native geometry conversion and preservation, inside the sketch executable.
using System;using System.IO;using System.Text;using System.Linq;using System.Collections.Generic;
using O=System.Collections.Generic.Dictionary<string,object>;
namespace SketchRust.Compat {
 public sealed partial class ApexFile {
  void ScaleUnits(double factor) {
   foreach(var page in Pages){
    foreach(var area in page.Areas){area.Height*=factor;area.Perimeter*=factor;area.Wall*=factor*factor;area.StoredArea*=factor*factor;}
    ScaleEntities(page.Entities,factor);foreach(var block in page.Blocks){block.X*=factor;block.Y*=factor;ScaleEntities(block.Entities,factor);}
    for(int i=0;i<page.Text.Count;i++){var raw=page.Text[i];using(var r=new R(raw,Version)){int token=r.ReadByte();var text=ReadText(r,token);text["x"]=Num(text,"x")*factor;text["y"]=Num(text,"y")*factor;text["fontSize"]=Num(text,"fontSize")*factor;text["id"]=Num(text,"id")+1;var area=page.Areas.FirstOrDefault(a=>a.SizeIndex>0&&a.SizeIndex==i);if(area!=null)text["text"]=area.StoredArea.ToString("0.##",System.Globalization.CultureInfo.InvariantCulture)+(factor<1?" m²":" sf");else if(Version>=9&&token==2){double length;string value=Str(text,"text").TrimEnd('\'','m');if(Double.TryParse(value,System.Globalization.NumberStyles.Float,System.Globalization.CultureInfo.InvariantCulture,out length))text["text"]=(length*factor).ToString("0.##",System.Globalization.CultureInfo.InvariantCulture)+(factor<1?"m":"'");}var record=TextRecord(text,Version);if(Version>=9&&token==2){record[0]=2;record=record.Concat(new byte[]{0}).ToArray();}page.Text[i]=record;}}
    if(Version>=37){var points=GeoPoints(page,Version);foreach(var point in points){point["x"]=Num(point,"x")*factor;point["y"]=Num(point,"y")*factor;}page.Attributes=StoreGeo(page,Obj("geoReferencePoints",points,"geoReferenceCoordinateSystem",GeoSystem(page,Version)),Version);}
   }
   if(Version>=36&&Tail.Length>4){using(var r=new R(Tail,Version)){var prefix=r.Bytes(4);r.Str();r.Str();var remainder=r.Bytes((int)(r.BaseStream.Length-r.BaseStream.Position));Tail=Pack(w=>{w.Write(prefix);w.Str(factor<1?" m²":" sf");w.Str(factor<1?"m":"'");w.Write(remainder);},Version);}}
  }
  static void ScaleEntities(IEnumerable<Ent> entities,double f){foreach(var e in entities){var v=e.Values;switch(e.Type){
   case 1:v[0]*=f;v[1]*=f;v[2]*=f;break;
   case 2:for(int i=0;i<4;i++)v[i]*=f;break;
   case 3:case 4:case 7:for(int i=0;i<3;i++)v[i]*=f;break;
   case 35:for(int i=0;i<6;i++)v[i]*=f;break;
   case 23:v[0]*=f;v[1]*=f;break;
   case 13:v[0]*=f;v[1]*=f;ScaleEntities(e.Children,f);break;
   case 14:for(int i=0;i<5;i++)v[i]*=f;break;
  }}}
  sealed class Affine {
   public double A=1,B,C,D=1,X,Y;
   public O Point(double x,double y){return Pt(A*x+C*y+X,B*x+D*y+Y);}
   public Affine Then(Affine q){return new Affine{A=A*q.A+C*q.B,B=B*q.A+D*q.B,C=A*q.C+C*q.D,D=B*q.C+D*q.D,X=A*q.X+C*q.Y+X,Y=B*q.X+D*q.Y+Y};}
  }
  static O EllipseGeometry(Ent e,Affine m){var v=e.Values;double ax,ay,bx,by,start,end;
   if(e.Type==35){ax=v[3];ay=v[4];double r=v[5]/Math.Max(1e-12,Hypot(ax,ay));bx=-ay*r;by=ax*r;start=v[6]*Math.PI/180;end=v[7]*Math.PI/180;}
   else{ax=v[2];ay=0;bx=0;by=v[2];start=e.Type==4?v[3]*Math.PI/180:0;end=e.Type==4?v[4]*Math.PI/180:2*Math.PI;}
   double ux=m.A*ax+m.C*ay,uy=m.B*ax+m.D*ay,vx=m.A*bx+m.C*by,vy=m.B*bx+m.D*by;
   double xx=ux*ux+vx*vx,yy=uy*uy+vy*vy,xy=ux*uy+vx*vy,disc=Hypot(xx-yy,2*xy),major=Math.Sqrt(Math.Max(0,(xx+yy+disc)/2)),minor=Math.Sqrt(Math.Max(0,(xx+yy-disc)/2)),angle=Math.Atan2(2*xy,xx-yy)/2;
   if(major<1e-12||minor<1e-12){var c=m.Point(v[0],v[1]);return Obj("entityType","LINE","startPoint",Inch(Num(c,"x")-major*Math.Cos(angle),Num(c,"y")-major*Math.Sin(angle)),"endPoint",Inch(Num(c,"x")+major*Math.Cos(angle),Num(c,"y")+major*Math.Sin(angle)));}double co=Math.Cos(angle),si=Math.Sin(angle);
   Func<double,double> parameter=t=>Math.Atan2((-(ux*Math.Cos(t)+vx*Math.Sin(t))*si+(uy*Math.Cos(t)+vy*Math.Sin(t))*co)/minor,((ux*Math.Cos(t)+vx*Math.Sin(t))*co+(uy*Math.Cos(t)+vy*Math.Sin(t))*si)/major);
   bool full=Math.Abs(end-start)>=2*Math.PI-1e-9;double first=parameter(start),last=parameter(end);if(ux*vy-uy*vx<0){double swap=first;first=last;last=swap;}while(last<=first+1e-12)last+=2*Math.PI;if(full){first=0;last=2*Math.PI;}
   var center=m.Point(v[0],v[1]);return Obj("entityType","ELLIPSE","center",Inch(Num(center,"x"),Num(center,"y")),"axisPoint",Inch(major*co,major*si),"ratio",minor/major,"startAngle",first*180/Math.PI,"endAngle",last*180/Math.PI);
  }
  List<O> Flatten(IEnumerable<Ent> entities,List<Block> blocks,Affine matrix,HashSet<string> visiting,int depth=0){
   if(depth>64)throw new InvalidDataException("Native block nesting limit exceeded");var result=new List<O>();foreach(var e in entities){var v=e.Values;O g=null;
    if(e.Type==23){if(!visiting.Add(e.Name))throw new InvalidDataException("Cyclic native block reference");var block=blocks.FirstOrDefault(b=>b.Name==e.Name);if(block==null)throw new InvalidDataException("Missing native block "+e.Name);double c=Math.Cos(v[4]*Math.PI/180),s=Math.Sin(v[4]*Math.PI/180);var local=new Affine{A=c*(v[2]==0?1:v[2]),B=s*(v[2]==0?1:v[2]),C=-s*(v[3]==0?1:v[3]),D=c*(v[3]==0?1:v[3])};local.X=v[0]-local.A*block.X-local.C*block.Y;local.Y=v[1]-local.B*block.X-local.D*block.Y;result.AddRange(Flatten(block.Entities,blocks,matrix.Then(local),visiting,depth+1));visiting.Remove(e.Name);continue;}
    if(e.Type==13){int count=(e.Flags&1)!=0?e.Children.Count:e.Children.Count-1;for(int i=0;i<count;i++){var a=e.Children[i].Values;var b=e.Children[(i+1)%e.Children.Count].Values;Ent segment;
      if(Math.Abs(a[5])<1e-12)segment=Entity(2,0,Version,a[0],a[1],b[0],b[1]);else{double dx=b[0]-a[0],dy=b[1]-a[1],cx=(a[0]+b[0])/2-dy*(1-a[5]*a[5])/(4*a[5]),cy=(a[1]+b[1])/2+dx*(1-a[5]*a[5])/(4*a[5]),first=Math.Atan2(a[1]-cy,a[0]-cx),sweep=4*Math.Atan(a[5]);segment=Entity(4,0,Version,cx,cy,Hypot(a[0]-cx,a[1]-cy),(sweep<0?first+sweep:first)*180/Math.PI,(sweep<0?first:first+sweep)*180/Math.PI);}segment.Prefix=e.Prefix;result.AddRange(Flatten(new[]{segment},blocks,matrix,visiting,depth+1));}continue;}
    if(e.Type==3||e.Type==4||e.Type==35)g=EllipseGeometry(e,matrix);
    else if(e.Type==2){var a=matrix.Point(v[0],v[1]);var b=matrix.Point(v[2],v[3]);g=Obj("entityType","LINE","startPoint",Inch(Num(a,"x"),Num(a,"y")),"endPoint",Inch(Num(b,"x"),Num(b,"y")));}
    else if(e.Type==1){var point=matrix.Point(v[0],v[1]);double x=Num(point,"x")*12,y=Num(point,"y")*12;foreach(var delta in new[]{new[]{-0.6,0.0,0.6,0.0},new[]{0.0,-0.6,0.0,0.6}})result.Add(Obj("entityType","LINE","startPoint",Pt(x+delta[0],y+delta[1]),"endPoint",Pt(x+delta[2],y+delta[3]),"strokeColor",EntityColor(e,Version),"nativePoint",true));continue;}
    else if(e.Type==7){var point=matrix.Point(v[0],v[1]);double rotation=BitConverter.ToDouble(e.Extras,0)*Math.PI/180,c=Math.Cos(rotation),s=Math.Sin(rotation);double size=v[2]*Hypot(-matrix.A*s+matrix.C*c,-matrix.B*s+matrix.D*c);g=Obj("entityType","TEXT","insertPoint",Inch(Num(point,"x"),Num(point,"y")),"fontSize",size*12,"textString",e.Text,"rotation",Math.Atan2(matrix.B*c+matrix.D*s,matrix.A*c+matrix.C*s)*180/Math.PI,"alignment","LEFT BOTTOM");}
    else throw new InvalidDataException("Unsupported native geometry entity "+e.Type);
    g["strokeColor"]=EntityColor(e,Version);result.Add(g);if(result.Count>100000)throw new InvalidDataException("Native expanded geometry limit exceeded");
   }return result;
  }
  O NativeDefinition(string name,Ent entity,NativePage page,out double x,out double y,out double rotation,out double scale){
   x=0;y=0;rotation=0;scale=1;
   if(entity.Type==23&&Math.Abs(entity.Values[2]-entity.Values[3])<1e-9){var record=ReadSymbolRecord(page,entity.Name);var block=page.Blocks.FirstOrDefault(b=>b.Name==entity.Name);if(record.Count>0&&block!=null&&Str(record,"blockHash")==BlockHash(block)){x=entity.Values[0];y=entity.Values[1];rotation=-entity.Values[4];scale=entity.Values[2]==0?1:entity.Values[2];var retained=Definition(name,Items(record,"geometry").ToList());retained["nativeApex"]=Get(record,"payload");return retained;}}// Retain records with the independent codec, never the vendor engine.
   Ent copy;using(var r=new R(Pack(w=>WriteEntities(w,new List<Ent>{entity}),Version),Version))copy=ReadEntities(r)[0];
   if(copy.Type==23){x=copy.Values[0];y=copy.Values[1];rotation=-copy.Values[4];copy.Values[0]=copy.Values[1]=copy.Values[4]=0;}
   else if(copy.Type==1){x=copy.Values[0];y=copy.Values[1];copy.Values[0]=copy.Values[1]=0;}
   var definition=Definition(name,Flatten(new[]{copy},page.Blocks,new Affine(),new HashSet<string>()));
   var needed=new List<Block>();var names=new HashSet<string>();var pending=new Stack<Ent>(new[]{copy});while(pending.Count>0){var item=pending.Pop();foreach(var child in item.Children)pending.Push(child);if(item.Type!=23||!names.Add(item.Name))continue;var block=page.Blocks.First(b=>b.Name==item.Name);needed.Add(block);foreach(var child in block.Entities)pending.Push(child);}
   definition["nativeApex"]=Obj("version",Version,"geometryHash",GeometryHash(Get(definition,"geometry")),"entities",Convert.ToBase64String(Pack(w=>WriteEntities(w,new List<Ent>{copy}),Version)),"blocks",Convert.ToBase64String(Pack(w=>WriteBlocks(w,needed),Version)));
   return definition;
  }
  static string GeometryHash(object value){var text=new System.Text.StringBuilder();Canonical(value,text);using(var sha=System.Security.Cryptography.SHA256.Create())return Convert.ToBase64String(sha.ComputeHash(System.Text.Encoding.UTF8.GetBytes(text.ToString())));}
  static void Canonical(object value,System.Text.StringBuilder output){
   var map=value as O;if(map!=null){output.Append('{');foreach(var pair in map.Where(p=>p.Value!=null).OrderBy(p=>p.Key,StringComparer.Ordinal)){output.Append(pair.Key.Length).Append(':').Append(pair.Key);Canonical(pair.Value,output);}output.Append('}');return;}
   var sequence=value as System.Collections.IEnumerable;if(sequence!=null&&!(value is string)){output.Append('[');foreach(var item in sequence)Canonical(item,output);output.Append(']');return;}
   if(value is string){string s=(string)value;output.Append('s').Append(s.Length).Append(':').Append(s);}
   else if(value is bool)output.Append((bool)value?"true":"false");else if(value==null)output.Append("null");else output.Append('n').Append(Math.Round(Convert.ToDouble(value,System.Globalization.CultureInfo.InvariantCulture),9).ToString("R",System.Globalization.CultureInfo.InvariantCulture));output.Append(';');
  }
  static void ConvertEntities(List<Ent> entities,int from,int to){foreach(var e in entities){int oldSize=from>=33?4:2,newSize=to>=33?4:2;int color=EntityColor(e,from);var prefix=e.Prefix;e.Prefix=Pack(w=>{w.Write(prefix.Take(40).ToArray());if(newSize==4)w.Write(color);else w.Write(PaletteIndex(color));w.Write(prefix.Skip(40+oldSize).ToArray());},to);if(e.Type==1)Array.Resize(ref e.Extras,to>=21?4:2);if(e.Type==7){Array.Resize(ref e.Extras,to>=38?27:24);if(from<38&&to>=38)e.Extras[25]=1;}ConvertEntities(e.Children,from,to);}}
  static Block RestoreNative(O payload,string prefix,NativePage page,int version,ref int key){int from=checked((int)Num(payload,"version"));if(from<1||from>38)throw new InvalidDataException("Invalid retained native record version");List<Ent> entities;List<Block> blocks;using(var r=new R(Convert.FromBase64String(Str(payload,"entities")),from))entities=ReadEntities(r);using(var r=new R(Convert.FromBase64String(Str(payload,"blocks")),from))blocks=ReadBlocks(r);
   var names=new Dictionary<string,string>();for(int i=0;i<blocks.Count;i++)names.Add(blocks[i].Name,prefix+"_"+i);var all=blocks.SelectMany(b=>b.Entities).Concat(entities).ToList();ConvertEntities(all,from,version);
   var pending=new Stack<Ent>(all);while(pending.Count>0){var e=pending.Pop();e.Key=key++;if(e.Type==23){string name;if(!names.TryGetValue(e.Name,out name))throw new InvalidDataException("Missing retained native block");e.Name=name;}foreach(var child in e.Children)pending.Push(child);}
   foreach(var b in blocks){b.Name=names[b.Name];Array.Resize(ref b.Tail,version>=38?21:version>=32?4:0);page.Blocks.Add(b);}return new Block{Name=prefix,Entities=entities,Tail=new byte[version>=38?21:version>=32?4:0]};
  }
  const string SymbolStateBlock="__SKETCH_RUST_SYMBOLS_V1";
  static O EntityState(Ent e){return Obj("type",e.Type,"prefix",Convert.ToBase64String(e.Prefix),"values",e.Values.Select(v=>Math.Round(v,8)).ToArray(),"extras",Convert.ToBase64String(e.Extras),"text",e.Text,"font",e.Font,"flags",e.Flags,"name",e.Name,"children",e.Children.Select(EntityState).ToArray());}
  static string BlockHash(Block block){return GeometryHash(Obj("x",Math.Round(block.X,8),"y",Math.Round(block.Y,8),"layer",block.Layer,"position",block.Position,"entities",block.Entities.Select(EntityState).ToArray()));}
  static O ReadSymbolRecord(NativePage page,string name){var state=page.Blocks.FirstOrDefault(b=>b.Name==SymbolStateBlock);if(state==null)return new O();string value=String.Concat(state.Entities.Select(e=>e.Text));if(value.Length>Limit)throw new InvalidDataException("Native symbol history exceeds the size limit");var records=ModelJson.Deserialize<O>(Encoding.UTF8.GetString(Convert.FromBase64String(value)));return Map(Get(records,name));}
  static void StoreSymbolRecords(NativePage page,O records,int version,ref int key){if(records.Count==0)return;string data=Convert.ToBase64String(Encoding.UTF8.GetBytes(ModelJson.Serialize(records)));var block=new Block{Name=SymbolStateBlock,Tail=new byte[version>=38?21:version>=32?4:0]};for(int i=0;i<data.Length;i+=60000){var text=Entity(7,key++,version,0,0,1);text.Text=data.Substring(i,Math.Min(60000,data.Length-i));text.Extras=new byte[version>=38?27:24];if(version>=38)text.Extras[25]=1;block.Entities.Add(text);}page.Blocks.Add(block);}
  List<Ent> NativePoints(IEnumerable<Ent> entities,List<Block> blocks,Affine matrix,HashSet<string> visiting,ref int key,int version){var points=new List<Ent>();foreach(var e in entities){var v=e.Values;if(e.Type==1){var p=matrix.Point(v[0],v[1]);var point=Entity(1,key++,version,Num(p,"x"),Num(p,"y"),0,0);point.Extras=new byte[version>=21?4:2];points.Add(point);}else if(e.Type==23){if(!visiting.Add(e.Name))throw new InvalidDataException("Cyclic native point transform");var block=blocks.First(b=>b.Name==e.Name);double c=Math.Cos(v[4]*Math.PI/180),s=Math.Sin(v[4]*Math.PI/180);var local=new Affine{A=c*v[2],B=s*v[2],C=-s*v[3],D=c*v[3]};local.X=v[0]-local.A*block.X-local.C*block.Y;local.Y=v[1]-local.B*block.X-local.D*block.Y;points.AddRange(NativePoints(block.Entities,blocks,matrix.Then(local),visiting,ref key,version));visiting.Remove(e.Name);}}return points;}
  Block EditableNativeBlock(O definition,O payload,string name,NativePage page,int version,ref int key){
   var recordsPage=new NativePage();var raw=RestoreNative(payload,name+"_RAW",recordsPage,version,ref key);var points=NativePoints(raw.Entities,recordsPage.Blocks,new Affine(),new HashSet<string>(),ref key,version);
   var entities=SymbolEntities(definition,version,ref key);entities.AddRange(points);return new Block{Name=name,Layer=points.Count==0?"MSA":"",Entities=entities,Tail=new byte[version>=38?21:version>=32?4:0]};
  }

 }
}
