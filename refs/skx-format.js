
/* Dedicated sketch file format (.skx).

   A .skx is a workfile container carrying a single part. It is deliberately
   the same ZIP shape as .apx, written by the same writer, and its one part
   lives at the same path the suite uses when it embeds the sketch in a full
   workfile:

       state/sketch.json

   The bytes at that path are the output of the sketch app's buildSaveModel()
   serialised with JSON.stringify and nothing else, so the standalone file and
   the embedded part are byte-identical for the same drawing. That is the whole
   point of the design: one model, one serialisation, two envelopes.

   The manifest keeps `format: appraisal-suite.workfile` so WorkfileContainer's
   own parseManifest accepts it and the shell's reader can open a .skx without
   special-casing; `profile: 'sketch'` and the mimetype entry are what tell the
   two file kinds apart. Per the format plan, readers identify by content — the
   mimetype entry and the ZIP magic — never by the extension.

   No blobs, no seal. The sketch model is pure vector geometry; a full
   three-page residential sketch is a few kilobytes deflated, and there is
   nothing in it to encrypt.

   Specification: docs/plans/WORKFILE_FORMAT_PLAN.md */

(function(root,factory){
  var api=factory(function(){
    if(root&&root.WorkfileContainer) return root.WorkfileContainer;
    if(typeof module==='object'&&module.exports){
      try{ return require('./workfile-container.js'); }catch(e){ return null; }
    }
    return null;
  });
  if(typeof module==='object'&&module.exports)module.exports=api;
  if(root)root.SketchFile=api;
})(typeof globalThis!=='undefined'?globalThis:this,function(getContainer){
  'use strict';

  var MIME_STRING  = 'application/vnd.appraisal-suite.sketch';
  var PROFILE      = 'sketch';
  var PART_PATH    = 'state/sketch.json';
  var SUMMARY_PATH = 'meta/summary.json';
  var EXTENSION    = '.skx';

  function container(){
    var wfc=getContainer();
    if(!wfc) throw new Error('SketchFile requires WorkfileContainer');
    return wfc;
  }

  /* ── summary ─────────────────────────────────────────────────────
     A small, always-plaintext part so a file list or a recovery UI can
     describe a sketch without inflating and parsing the whole model. */
  function buildSummary(model){
    var pages=(model&&Array.isArray(model.pages))?model.pages:[];
    var counts=pages.reduce(function(acc,page){
      acc.lines   += (page&&Array.isArray(page.lines))?page.lines.length:0;
      acc.polygons+= (page&&Array.isArray(page.polygons))?page.polygons.length:0;
      acc.labels  += (page&&Array.isArray(page.labels))?page.labels.length:0;
      acc.symbols += (page&&Array.isArray(page.symbols))?page.symbols.length:0;
      return acc;
    },{lines:0,polygons:0,labels:0,symbols:0});

    /* subjectInfo has been both a string and an object across versions; the
       summary must not throw on either. */
    var address='';
    var info=model&&model.subjectInfo;
    if(info&&typeof info==='object') address=info.propertyAddress||'';
    else if(typeof info==='string') address=info;

    return {
      profile:PROFILE,
      fileName:(model&&model.fileName)||'Untitled',
      propertyAddress:address,
      pages:pages.length,
      counts:counts,
      savedAt:(model&&model.savedAt)||new Date().toISOString()
    };
  }

  /* ── write ───────────────────────────────────────────────────────
     Returns a Blob. `model` is the buildSaveModel() output, untouched. */
  function write(model,opts){
    var wfc=container();
    opts=opts||{};
    if(!model||!Array.isArray(model.pages)) return Promise.reject(new Error('SketchFile.write: model has no pages'));

    var json=JSON.stringify(model);
    var summary=JSON.stringify(buildSummary(model));
    var enc=new TextEncoder();
    var partBytes=enc.encode(json);
    var summaryBytes=enc.encode(summary);

    return Promise.all([wfc.sha256Hex(partBytes),wfc.sha256Hex(summaryBytes)]).then(function(hashes){
      var manifest=wfc.buildManifest({
        schema:opts.schema||1,
        assignmentUid:opts.assignmentUid||'',
        savedAt:(model&&model.savedAt)||new Date().toISOString(),
        parts:[
          {path:SUMMARY_PATH,role:'meta.summary',bytes:summaryBytes.length,sha256:hashes[1],encoding:'json'},
          {path:PART_PATH,   role:'app.sketch',  bytes:partBytes.length,   sha256:hashes[0],encoding:'json'}
        ],
        blobs:[],
        protection:{mode:'open'}
      });
      manifest.profile=PROFILE;

      return wfc.writeZip([
        {path:'mimetype',      data:MIME_STRING,             method:'store'},
        {path:'manifest.json', data:JSON.stringify(manifest),method:'deflate'},
        {path:SUMMARY_PATH,    data:summary,                 method:'deflate'},
        {path:PART_PATH,       data:json,                    method:'deflate'}
      ]);
    });
  }

  /* ── read ────────────────────────────────────────────────────────
     Accepts a .skx, and also any workfile container that happens to carry a
     state/sketch.json part — so a full .apx can be opened directly by the
     sketch app and yield just its sketch. Returns {model, manifest, summary}. */
  function read(blob){
    var wfc=container();
    return wfc.readZipEntries(blob).then(function(entries){
      var byPath={};
      entries.forEach(function(e){ byPath[e.path]=e; });

      var part=byPath[PART_PATH];
      if(!part) throw new Error('Not a sketch file: no ' + PART_PATH + ' part');

      var reads=[wfc.readEntry(blob,part)];
      reads.push(byPath['manifest.json']?wfc.readEntry(blob,byPath['manifest.json']):null);
      reads.push(byPath[SUMMARY_PATH]  ?wfc.readEntry(blob,byPath[SUMMARY_PATH]  ):null);

      return Promise.all(reads).then(function(bufs){
        var dec=new TextDecoder();
        var model=JSON.parse(dec.decode(new Uint8Array(bufs[0])));
        var manifest=null,summary=null;
        /* A damaged or absent manifest must not cost the appraiser their
           drawing: the model part carries its own CRC and its own hash, and it
           is the only part that holds content. */
        try{ if(bufs[1]) manifest=wfc.parseManifest(dec.decode(new Uint8Array(bufs[1]))); }catch(e){ manifest=null; }
        try{ if(bufs[2]) summary=JSON.parse(dec.decode(new Uint8Array(bufs[2]))); }catch(e){ summary=null; }
        return {model:model,manifest:manifest,summary:summary};
      });
    });
  }

  /* ── sniff ───────────────────────────────────────────────────────
     Read a file without knowing which format it is. ZIP magic selects the
     container path; anything else is parsed as a legacy .json sketch. Resolves
     {model, container:boolean}. */
  function readAny(blob){
    return blob.slice(0,4).arrayBuffer().then(function(head){
      var bytes=new Uint8Array(head);
      if(bytes.length>=4&&container().isZipBytes(bytes)){
        return read(blob).then(function(result){
          return {model:result.model,manifest:result.manifest,summary:result.summary,container:true};
        });
      }
      return blob.text().then(function(text){
        return {model:JSON.parse(text),manifest:null,summary:null,container:false};
      });
    });
  }

  return{
    write:write,
    read:read,
    readAny:readAny,
    buildSummary:buildSummary,
    MIME_STRING:MIME_STRING,
    PROFILE:PROFILE,
    PART_PATH:PART_PATH,
    SUMMARY_PATH:SUMMARY_PATH,
    EXTENSION:EXTENSION
  };
});
  