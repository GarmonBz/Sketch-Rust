
/* Minimal ZIP container for .apx workfile format.
   Uses CompressionStream/DecompressionStream for deflate-raw (ZIP method 8).
   No external dependency; roughly 300 lines.
   Specification: docs/plans/WORKFILE_FORMAT_PLAN.md */

(function(root,factory){
  var api=factory();
  if(typeof module==='object'&&module.exports)module.exports=api;
  if(root)root.WorkfileContainer=api;
})(typeof globalThis!=='undefined'?globalThis:this,function(){
  'use strict';

  /* ── constants ──────────────────────────────────────────────── */
  var LOCAL_SIG     = 0x04034b50;
  var CENTRAL_SIG   = 0x02014b50;
  var EOCD_SIG      = 0x06054b50;
  var EOCD_SIZE     = 22;
  var METHOD_STORE  = 0;
  var METHOD_DEFLATE= 8;
  var MIN_VERSION   = 20;        // 2.0 (deflate support)
  var FORMAT        = 'appraisal-suite.workfile';
  var FORMAT_VERSION= 1;
  var MIME_STRING   = 'application/vnd.appraisal-suite.workfile';

  /* ── CRC-32 table (lazy) ────────────────────────────────────── */
  var crcTable = null;
  function initCrcTable(){
    crcTable = new Uint32Array(256);
    for(var i=0;i<256;i++){
      var c=i;
      for(var j=0;j<8;j++) c=(c&1)?(0xEDB88320^(c>>>1)):(c>>>1);
      crcTable[i]=c;
    }
  }

  function crc32(data){
    if(!crcTable) initCrcTable();
    /* A string must be measured as the UTF-8 bytes the archive will hold.
       Hashing UTF-16 code units instead would silently disagree with the
       stored entry for any non-ASCII content and fail verification on read. */
    if(typeof data==='string') data=strBytes(data);
    var crc=0xFFFFFFFF;
    for(var i=0;i<data.length;i++) crc=crcTable[(crc^data[i])&0xFF]^(crc>>>8);
    return (crc^0xFFFFFFFF)>>>0;
  }

  /* ── uint helpers ────────────────────────────────────────────── */
  function u32LE(v){ var d=new DataView(new ArrayBuffer(4)); d.setUint32(0,v,true); return new Uint8Array(d.buffer); }
  function u16LE(v){ var d=new DataView(new ArrayBuffer(2)); d.setUint16(0,v,true); return new Uint8Array(d.buffer); }
  function readU16(dv,off){ return dv.getUint16(off,true); }
  function readU32(dv,off){ return dv.getUint32(off,true); }

  /* ── compression ─────────────────────────────────────────────── */
  function deflateBytes(bytes){
    var cs=new CompressionStream('deflate-raw');
    var writer=cs.writable.getWriter();
    writer.write(bytes);
    writer.close();
    return new Response(cs.readable).arrayBuffer();
  }

  function inflateBytes(compressed,uncompressedSize){
    var ds=new DecompressionStream('deflate-raw');
    var writer=ds.writable.getWriter();
    writer.write(compressed);
    writer.close();
    return new Response(ds.readable).arrayBuffer();
  }

  /* ── string <-> bytes ────────────────────────────────────────── */
  function strBytes(s){ return new TextEncoder().encode(s); }
  function bytesStr(b){ return new TextDecoder().decode(b); }

  /* ── SHA-256 ─────────────────────────────────────────────────── */
  /* WebCrypto exists only in a secure context (https or localhost); the suite
     is also served over plain http on a LAN address, where crypto.subtle is
     undefined. The same digest is computed in script there. */
  var SHA_K=[0x428a2f98,0x71374491,0xb5c0fbcf,0xe9b5dba5,0x3956c25b,0x59f111f1,0x923f82a4,0xab1c5ed5,0xd807aa98,0x12835b01,0x243185be,0x550c7dc3,0x72be5d74,0x80deb1fe,0x9bdc06a7,0xc19bf174,0xe49b69c1,0xefbe4786,0x0fc19dc6,0x240ca1cc,0x2de92c6f,0x4a7484aa,0x5cb0a9dc,0x76f988da,0x983e5152,0xa831c66d,0xb00327c8,0xbf597fc7,0xc6e00bf3,0xd5a79147,0x06ca6351,0x14292967,0x27b70a85,0x2e1b2138,0x4d2c6dfc,0x53380d13,0x650a7354,0x766a0abb,0x81c2c92e,0x92722c85,0xa2bfe8a1,0xa81a664b,0xc24b8b70,0xc76c51a3,0xd192e819,0xd6990624,0xf40e3585,0x106aa070,0x19a4c116,0x1e376c08,0x2748774c,0x34b0bcb5,0x391c0cb3,0x4ed8aa4a,0x5b9cca4f,0x682e6ff3,0x748f82ee,0x78a5636f,0x84c87814,0x8cc70208,0x90befffa,0xa4506ceb,0xbef9a3f7,0xc67178f2];
  function sha256Script(bytes){
    var data=bytes instanceof Uint8Array?bytes:new Uint8Array(bytes.buffer?bytes.buffer.slice(bytes.byteOffset,bytes.byteOffset+bytes.byteLength):bytes);
    var len=data.length,total=((len+9+63)>>6)<<6,buf=new Uint8Array(total);
    buf.set(data);buf[len]=0x80;
    var view=new DataView(buf.buffer);
    view.setUint32(total-8,Math.floor(len/0x20000000),false);
    view.setUint32(total-4,(len<<3)>>>0,false);
    var H=[0x6a09e667,0xbb67ae85,0x3c6ef372,0xa54ff53a,0x510e527f,0x9b05688c,0x1f83d9ab,0x5be0cd19],W=new Array(64);
    function rr(x,n){return (x>>>n)|(x<<(32-n));}
    for(var off=0;off<total;off+=64){
      for(var i=0;i<16;i++)W[i]=view.getUint32(off+i*4,false);
      for(i=16;i<64;i++){
        var s0=rr(W[i-15],7)^rr(W[i-15],18)^(W[i-15]>>>3),s1=rr(W[i-2],17)^rr(W[i-2],19)^(W[i-2]>>>10);
        W[i]=(W[i-16]+s0+W[i-7]+s1)|0;
      }
      var a=H[0],b=H[1],c=H[2],d=H[3],e=H[4],f=H[5],g=H[6],h=H[7];
      for(i=0;i<64;i++){
        var t1=(h+(rr(e,6)^rr(e,11)^rr(e,25))+((e&f)^(~e&g))+SHA_K[i]+W[i])|0;
        var t2=((rr(a,2)^rr(a,13)^rr(a,22))+((a&b)^(a&c)^(b&c)))|0;
        h=g;g=f;f=e;e=(d+t1)|0;d=c;c=b;b=a;a=(t1+t2)|0;
      }
      H[0]=(H[0]+a)|0;H[1]=(H[1]+b)|0;H[2]=(H[2]+c)|0;H[3]=(H[3]+d)|0;H[4]=(H[4]+e)|0;H[5]=(H[5]+f)|0;H[6]=(H[6]+g)|0;H[7]=(H[7]+h)|0;
    }
    return H.map(function(x){return ('00000000'+(x>>>0).toString(16)).slice(-8);}).join('');
  }
  function sha256Hex(bytes){
    var subtle=typeof crypto!=='undefined'&&crypto&&crypto.subtle;
    if(!subtle)return Promise.resolve(sha256Script(bytes));
    return subtle.digest('SHA-256',bytes).then(function(h){
      return Array.prototype.map.call(new Uint8Array(h),function(x){return x.toString(16).padStart(2,'0');}).join('');
    });
  }
  var computeRefHash=sha256Hex;

  /* ═══════════════════════════════════════════════════════════════
     ZIP WRITER
     ═══════════════════════════════════════════════════════════════ */

  /**
   * Write a ZIP archive from an array of entries.
   *
   * Entry shape:
   *   { path: string, data: ArrayBuffer|Uint8Array|string,
   *     method: 'store'|'deflate' (default deflate) }
   *
   * Returns a Blob (lazy-assembled from slices).
   */
  function writeZip(entries){
    if(!entries||!entries.length) throw new Error('writeZip: at least one entry required (mimetype)');
    if(entries.length>65535) throw new Error('ZIP64 not supported: too many entries (>65535)');
    if(entries[0].path!=='mimetype') throw new Error('writeZip: first entry must be mimetype');

    return Promise.all(entries.map(function(e){
      return prepareEntry(e);
    })).then(function(prepared){
      var slices=[];
      var central=[];
      var offset=0;
      var totalSize=0;

      prepared.forEach(function(p,i){
        var local=buildLocalHeader(p,offset);
        slices.push(local.header);
        if(p.compressed){ slices.push(p.compressed); }
        else { slices.push(new Uint8Array(0)); }
        var entryLen=local.header.byteLength+(p.compressed?p.compressed.byteLength:0);
        central.push(buildCentralEntry(p,offset,i));
        offset+=entryLen;
        totalSize+=entryLen;
      });

      var cdStart=offset;
      central.forEach(function(c){ slices.push(c); totalSize+=c.byteLength; });
      var cdEnd=totalSize;
      var cdBytes=cdEnd-cdStart;

      var eocd=eocdRecord(entries.length,cdStart,cdBytes);
      slices.push(eocd);

      if(totalSize+eocd.byteLength>4294967295) throw new Error('ZIP64 not supported: archive exceeds 4 GB');
      return new Blob(slices,{type:'application/octet-stream'});
    });
  }

  function prepareEntry(e){
    return Promise.resolve().then(function(){
      var data;
      if(typeof e.data==='string') data=strBytes(e.data);
      else if(e.data instanceof ArrayBuffer) data=new Uint8Array(e.data);
      else if(e.data instanceof Uint8Array) data=e.data;
      else throw new Error('Unsupported entry data type for '+e.path);
      var method=(e.method==='store')?METHOD_STORE:METHOD_DEFLATE;
      var crc=crc32(data);
      var uncompressed=data.byteLength;

      if(method===METHOD_STORE){
        return {path:e.path,method:method,crc:crc,uncompressed:uncompressed,compressed:data};
      }
      // deflate
      return deflateBytes(data).then(function(compressedAB){
        var compressed=new Uint8Array(compressedAB);
        return {path:e.path,method:method,crc:crc,uncompressed:uncompressed,compressed:compressed};
      });
    });
  }

  function buildLocalHeader(p,offset){
    var nameBytes=strBytes(p.path);
    var nameLen=nameBytes.length;
    // flags bit 3 (data descriptor) NOT set; we know sizes
    var flags=0;
    var hdrSize=30+nameLen;
    var buf=new ArrayBuffer(hdrSize);
    var dv=new DataView(buf);
    dv.setUint32(0,LOCAL_SIG,true);
    dv.setUint16(4,MIN_VERSION,true);
    dv.setUint16(6,flags,true);
    dv.setUint16(8,p.method,true);
    dv.setUint16(10,0,true);  // mod time (unused)
    dv.setUint16(12,0,true);  // mod date (unused)
    dv.setUint32(14,p.crc,true);
    dv.setUint32(18,p.compressed.byteLength,true);
    dv.setUint32(22,p.uncompressed,true);
    dv.setUint16(26,nameLen,true);
    dv.setUint16(28,0,true);  // extra field length
    var hdr=new Uint8Array(buf);
    hdr.set(nameBytes,30);
    return {header:hdr,offset:offset};
  }

  function buildCentralEntry(p,localOffset,index){
    var nameBytes=strBytes(p.path);
    var nameLen=nameBytes.length;
    var buf=new ArrayBuffer(46+nameLen);
    var dv=new DataView(buf);
    dv.setUint32(0,CENTRAL_SIG,true);
    dv.setUint16(4,MIN_VERSION,true);   // version made by
    dv.setUint16(6,MIN_VERSION,true);   // version needed
    dv.setUint16(8,0,true);             // flags
    dv.setUint16(10,p.method,true);
    dv.setUint16(12,0,true);            // mod time
    dv.setUint16(14,0,true);            // mod date
    dv.setUint32(16,p.crc,true);
    dv.setUint32(20,p.compressed.byteLength,true);
    dv.setUint32(24,p.uncompressed,true);
    dv.setUint16(28,nameLen,true);
    dv.setUint16(30,0,true);            // extra field length
    dv.setUint16(32,0,true);            // file comment length
    dv.setUint16(34,0,true);            // disk number start
    dv.setUint16(36,0,true);            // internal attributes
    dv.setUint32(38,0,true);            // external attributes
    dv.setUint32(42,localOffset,true);
    var entry=new Uint8Array(buf);
    entry.set(nameBytes,46);
    return entry;
  }

  function eocdRecord(numEntries,cdOffset,cdSize){
    var buf=new ArrayBuffer(EOCD_SIZE);
    var dv=new DataView(buf);
    dv.setUint32(0,EOCD_SIG,true);
    dv.setUint16(4,0,true);             // disk number
    dv.setUint16(6,0,true);             // disk with CD
    dv.setUint16(8,numEntries,true);    // entries on this disk
    dv.setUint16(10,numEntries,true);   // total entries
    dv.setUint32(12,cdSize,true);
    dv.setUint32(16,cdOffset,true);
    dv.setUint16(20,0,true);            // comment length
    return new Uint8Array(buf);
  }

  /* ═══════════════════════════════════════════════════════════════
     ZIP READER
     ═══════════════════════════════════════════════════════════════ */

  /**
   * Parse a ZIP blob and return an array of entry descriptors.
   * Each entry: { path, method, crc, compressedSize, uncompressedSize,
   *               localOffset }
   */
  function readZipEntries(blob){
    return readTail(blob).then(function(eocd){
      // Read central directory
      var cdSize=eocd.cdSize;
      var cdOffset=eocd.cdOffset;
      var totalEntries=eocd.totalEntries;
      return blob.slice(cdOffset,cdOffset+cdSize).arrayBuffer().then(function(cdBuf){
        var entries=[];
        var dv=new DataView(cdBuf);
        var off=0;
        for(var i=0;i<totalEntries;i++){
          if(dv.getUint32(off,true)!==CENTRAL_SIG) throw new Error('Corrupt ZIP: central directory signature mismatch');
          var nameLen=readU16(dv,off+28);
          var extraLen=readU16(dv,off+30);
          var commentLen=readU16(dv,off+32);
          var nameBytes=new Uint8Array(cdBuf,off+46,nameLen);
          var path=bytesStr(nameBytes);
          entries.push({
            path:path,
            method:readU16(dv,off+10),
            crc:readU32(dv,off+16),
            compressedSize:readU32(dv,off+20),
            uncompressedSize:readU32(dv,off+24),
            localOffset:readU32(dv,off+42)
          });
          off+=46+nameLen+extraLen+commentLen;
        }
        return entries;
      });
    });
  }

  /** Read the EOCD record from the tail of the blob. */
  function readTail(blob){
    var tailSize=Math.min(blob.size,EOCD_SIZE+65535); // max comment length
    var tailOffset=blob.size-tailSize;
    return blob.slice(tailOffset).arrayBuffer().then(function(buf){
      var bytes=new Uint8Array(buf);
      // Search backward for EOCD signature
      for(var i=bytes.length-EOCD_SIZE;i>=0;i--){
        if(bytes[i]===0x50&&bytes[i+1]===0x4b&&bytes[i+2]===0x05&&bytes[i+3]===0x06){
          var dv=new DataView(buf,i);
          return {
            diskNum:readU16(dv,4),
            cdDisk:readU16(dv,6),
            entriesDisk:readU16(dv,8),
            totalEntries:readU16(dv,10),
            cdSize:readU32(dv,12),
            cdOffset:readU32(dv,16),
            commentLen:readU16(dv,20)
          };
        }
      }
      throw new Error('Not a valid ZIP archive (no EOCD signature)');
    });
  }

  /**
   * Read and decompress a single entry from a ZIP blob.
   * Returns an ArrayBuffer of the decompressed data.
   */
  function readEntry(blob,entry){
    /* Read the fixed 30-byte local header first, then slice exactly the
       entry's payload. Slicing to the end of the blob would pull the whole
       remainder of the container into memory for every entry read, which
       defeats the random access the central directory exists to provide and
       makes opening an N-part workfile cost O(N x filesize). */
    return blob.slice(entry.localOffset,entry.localOffset+30).arrayBuffer().then(function(hdrBuf){
      if(hdrBuf.byteLength<30) throw new Error('Corrupt workfile: truncated local header at offset '+entry.localOffset);
      var dv=new DataView(hdrBuf);
      if(dv.getUint32(0,true)!==LOCAL_SIG) throw new Error('Corrupt workfile: local header signature mismatch at offset '+entry.localOffset);
      var nameLen=readU16(dv,26);
      var extraLen=readU16(dv,28);
      var dataStart=entry.localOffset+30+nameLen+extraLen;
      return blob.slice(dataStart,dataStart+entry.compressedSize).arrayBuffer();
    }).then(function(dataBuf){
      if(dataBuf.byteLength!==entry.compressedSize) throw new Error('Corrupt workfile: short read for '+entry.path);
      if(entry.method===METHOD_STORE) return dataBuf;
      if(entry.method===METHOD_DEFLATE) return inflateBytes(new Uint8Array(dataBuf),entry.uncompressedSize);
      throw new Error('Unsupported ZIP compression method: '+entry.method);
    }).then(function(plainBuf){
      /* Per-entry integrity. The stored CRC-32 is the container's cheapest
         corruption check; at ~480 MB/s it is negligible beside the read. */
      var bytes=new Uint8Array(plainBuf);
      if(bytes.length!==entry.uncompressedSize) throw new Error('Corrupt workfile: size mismatch for '+entry.path);
      if(crc32(bytes)!==entry.crc) throw new Error('Corrupt workfile: CRC mismatch for '+entry.path);
      return plainBuf;
    });
  }

  /* ═══════════════════════════════════════════════════════════════
     MANIFEST
     ═══════════════════════════════════════════════════════════════ */

  /**
   * Build the manifest.json object for an .apx container.
   * @param {object} opts
   *   - schema: workfile schema version (default 1)
   *   - assignmentUid: string
   *   - savedAt: ISO string
   *   - parts: array of {path, role, bytes, sha256, encoding}
   *   - blobs: array of {sha256, bytes, mime, refs}
   *   - protection: {mode:'open'} (or sealed)
   */
  function buildManifest(opts){
    var manifest={
      format:FORMAT,
      formatVersion:FORMAT_VERSION,
      schema:opts.schema||1,
      generator:{app:'Appraisal Suite',build:''},
      savedAt:opts.savedAt||new Date().toISOString(),
      assignmentUid:opts.assignmentUid||'',
      parts:(opts.parts||[]).map(function(p){ return {path:p.path,role:p.role,bytes:p.bytes,sha256:p.sha256,encoding:p.encoding||'json'}; }),
      blobs:(opts.blobs||[]).map(function(b){ return {sha256:b.sha256,bytes:b.bytes,mime:b.mime,refs:b.refs||1}; }),
      protection:opts.protection||{mode:'open'}
    };
    return manifest;
  }

  function parseManifest(data){
    if(typeof data==='string') data=JSON.parse(data);
    if(data.format!==FORMAT) throw new Error('Not an Appraisal Suite workfile (format: '+data.format+')');
    return data;
  }

  /* ═══════════════════════════════════════════════════════════════
     BLOB EXTRACTION / REHYDRATION  (Phase 2)
     ═══════════════════════════════════════════════════════════════ */

  var BLOB_REF_RE=/^blob:sha256-([0-9a-f]{64})$/;
  var DATA_URL_RE=/^data:image\/([^;]+);base64,(.+)$/;

  /**
   * Extract blobs from state object (async). Replaces data:image URLs
   * with blob:sha256-<hex> reference strings.
   * Returns {blobMap:{sha256->{bytes,mime,refs}}, state:modifiedState}
   */
  function extractBlobsAsync(state){
    /* Collect distinct encodings only. The same photograph routinely appears
       in several report profiles and galleries, and decoding plus hashing it
       once per occurrence is pure waste on image-heavy assignments. A Map
       keeps the large base64 keys out of a plain object's property table. */
    var unique=new Map();          /* base64 -> {mime, refs} */
    function collect(obj){
      if(!obj||typeof obj!=='object') return;
      if(Array.isArray(obj)){ obj.forEach(collect); return; }
      Object.keys(obj).forEach(function(k){
        var v=obj[k];
        if(typeof v!=='string'){ if(v&&typeof v==='object') collect(v); return; }
        var m=v.match(DATA_URL_RE);
        if(!m) return;
        var rec=unique.get(m[2]);
        if(rec) rec.refs++;
        else unique.set(m[2],{mime:'image/'+m[1],refs:1});
      });
    }
    collect(state);

    var encodings=Array.from(unique.keys());
    return Promise.all(encodings.map(function(b64){
      var rec=unique.get(b64),raw;
      try{ raw=atob(b64); }catch(e){ raw=''; }
      var bytes=new Uint8Array(raw.length);
      for(var i=0;i<raw.length;i++) bytes[i]=raw.charCodeAt(i);
      return sha256Hex(bytes).then(function(hash){
        return {b64:b64,bytes:bytes,hash:hash,mime:rec.mime,refs:rec.refs};
      });
    })).then(function(results){
      var blobMap={};
      var b64ToHash=new Map();
      results.forEach(function(r){
        /* Two different encodings can still decode to identical bytes; the
           content hash is the identity, so merge them and sum references. */
        if(!blobMap[r.hash]) blobMap[r.hash]={bytes:r.bytes,mime:r.mime,refs:0};
        blobMap[r.hash].refs+=r.refs;
        b64ToHash.set(r.b64,r.hash);
      });

      var modified=JSON.parse(JSON.stringify(state));
      function replaceBlobs(obj){
        if(!obj||typeof obj!=='object') return;
        if(Array.isArray(obj)){ obj.forEach(replaceBlobs); return; }
        Object.keys(obj).forEach(function(k){
          var v=obj[k];
          if(typeof v!=='string'){ if(v&&typeof v==='object') replaceBlobs(v); return; }
          var m=v.match(DATA_URL_RE);
          if(!m) return;
          var hash=b64ToHash.get(m[2]);
          if(hash) obj[k]='blob:sha256-'+hash;
        });
      }
      replaceBlobs(modified);

      return {blobMap:blobMap,state:modified};
    });
  }

  /**
   * Rehydrate blob references in a state object back to data URLs.
   * @param {object} state - the state object with blob:sha256-... refs
   * @param {object} blobMap - {sha256: {bytes: Uint8Array, mime: string}}
   */
  function rehydrateBlobs(state,blobMap){
    var modified=JSON.parse(JSON.stringify(state));

    function walk(obj){
      if(!obj||typeof obj!=='object') return;
      if(Array.isArray(obj)){ obj.forEach(walk); return; }
      Object.keys(obj).forEach(function(k){
        var v=obj[k];
        if(typeof v!=='string'){ if(v&&typeof v==='object') walk(v); return; }
        var m=v.match(BLOB_REF_RE);
        if(!m) return;
        var sha=m[1];
        var blob=blobMap[sha];
        if(!blob) return; // orphan reference — leave as-is
        var b64=bytesToB64(blob.bytes);
        obj[k]='data:'+blob.mime+';base64,'+b64;
      });
    }

    walk(modified);
    return modified;
  }

  function bytesToB64(bytes){
    var raw='',step=32768;
    for(var i=0;i<bytes.length;i+=step){
      raw+=String.fromCharCode.apply(null,bytes.subarray(i,Math.min(bytes.length,i+step)));
    }
    return btoa(raw);
  }

  /* ── sniff ZIP magic ─────────────────────────────────────────── */
  function isZipBlob(blob){ return blob&&blob.size>=4; }
  function isZipBytes(firstBytes){
    return firstBytes[0]===0x50&&firstBytes[1]===0x4b&&firstBytes[2]===0x03&&firstBytes[3]===0x04;
  }

  /* ── public API ──────────────────────────────────────────────── */
  return{
    crc32:crc32,
    writeZip:writeZip,
    readZipEntries:readZipEntries,
    readEntry:readEntry,
    buildManifest:buildManifest,
    parseManifest:parseManifest,
    extractBlobsAsync:extractBlobsAsync,
    rehydrateBlobs:rehydrateBlobs,
    isZipBytes:isZipBytes,
    sha256Hex:sha256Hex,
    sha256Script:sha256Script,
    // constants
    FORMAT:FORMAT,
    FORMAT_VERSION:FORMAT_VERSION,
    MIME_STRING:MIME_STRING,
    METHOD_STORE:METHOD_STORE,
    METHOD_DEFLATE:METHOD_DEFLATE
  };
});
  