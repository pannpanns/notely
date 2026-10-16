/* Escaped Markdown subset: no raw HTML, embedded images or executable links. */
function cleanSummaryText(value){
 let started=false;
 return String(value||'').split('\n').filter(line=>{
  const plain=line.replace(/^[>#*\s]+/,'').trim();
  if(!started && (!plain || /^[-_*]{3,}$/.test(plain)))return false;
  if(!started && (/^(Dibuat oleh |Sumber AI:)/.test(plain)||/^(?:Berikut(?: ini)? adalah|Berikut)\s+(?:ringkasan|catatan)\b/i.test(plain)))return false;
  // Older saved summaries put their attribution immediately below the title.
  if(/^>\s*Dibuat oleh .+Periksa kembali dengan transkrip asli\.$/.test(line.trim()))return false;
  if(/^#\s/.test(line))return true;
  started=true;return true;
 }).join('\n').trim();
}
function renderSummaryMarkdown(value){
 const escape=s=>String(s).replaceAll('&','&amp;').replaceAll('<','&lt;').replaceAll('>','&gt;').replaceAll('"','&quot;').replaceAll("'",'&#39;');
 const inline=s=>escape(s).replace(/`([^`]+)`/g,'<code>$1</code>').replace(/\*\*(.+?)\*\*/g,'<strong>$1</strong>').replace(/\*([^*]+)\*/g,'<em>$1</em>');
 let html='',list='',fence=false,code=[];
 const close=()=>{if(list){html+=`</${list}>`;list='';}};
 for(const line of cleanSummaryText(value).split('\n')){
  if(/^\s*```/.test(line)){close();if(fence){html+='<pre><code>'+escape(code.join('\n'))+'</code></pre>';code=[];}fence=!fence;continue;}
  if(fence){code.push(line);continue;}
  if(!line.trim()){close();continue;}
  const heading=line.match(/^\s{0,3}(#{1,6})\s+(.+?)\s*#*$/);
  if(heading){close();let n=heading[1].length;html+=`<h${n}>${inline(heading[2])}</h${n}>`;continue;}
  if(/^\s*(?:-{3,}|\*{3,}|_{3,})\s*$/.test(line)){close();html+='<hr>';continue;}
  const bullet=line.match(/^\s*(?:([-*+])\s+|(\d+)[.)]\s+)(.*)$/);
  if(bullet){const tag=bullet[1]?'ul':'ol';if(list!==tag){close();list=tag;html+=tag==='ol'?`<ol start="${Number(bullet[2])}">`:'<ul>';}html+='<li>'+inline(bullet[3])+'</li>';continue;}
  close();html+=/^>\s?/.test(line)?'<blockquote>'+inline(line.replace(/^>\s?/,''))+'</blockquote>':'<p>'+inline(line)+'</p>';
 }
 close();if(fence)html+='<pre><code>'+escape(code.join('\n'))+'</code></pre>';
 return html;
}
if(typeof module!=='undefined')module.exports={cleanSummaryText,renderSummaryMarkdown};
