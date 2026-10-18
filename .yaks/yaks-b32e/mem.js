// A/B: open a farm in openvscode-server, expand .yaks, report renderer memory + DOM nodes.
const puppeteer=require('puppeteer-core');
const folder=process.argv[2], label=process.argv[3];
const sleep=ms=>new Promise(r=>setTimeout(r,ms));
(async()=>{
  const b=await puppeteer.launch({executablePath:'/opt/vivaldi/vivaldi',headless:'new',
    args:['--no-sandbox','--disable-gpu','--disable-dev-shm-usage','--user-data-dir=/var/tmp/yakscale/ide/cd-'+label],
    defaultViewport:{width:1600,height:1000},protocolTimeout:900000});
  const p=await b.newPage();
  const t0=Date.now();
  await p.goto('http://127.0.0.1:9888/?folder='+encodeURIComponent(folder),{waitUntil:'domcontentloaded',timeout:120000});
  await p.waitForSelector('.monaco-workbench',{timeout:120000});
  await p.waitForFunction(()=>[...document.querySelectorAll('.explorer-folders-view .monaco-list-row')]
    .some(r=>(r.innerText||'').trim().startsWith('.yaks')),{timeout:120000,polling:100});
  const tExplorer=Date.now()-t0;
  await sleep(1500);
  await p.evaluate(()=>{const r=[...document.querySelectorAll('.explorer-folders-view .monaco-list-row')]
    .find(x=>(x.innerText||'').trim().startsWith('.yaks'));
    ['mousedown','mouseup','click'].forEach(e=>r.dispatchEvent(new MouseEvent(e,{bubbles:true,cancelable:true,button:0})));});
  await sleep(500);
  const tc=Date.now();
  await p.keyboard.press('ArrowRight');
  let expanded=false;
  try{ await p.waitForFunction(()=>[...document.querySelectorAll('.explorer-folders-view .monaco-list-row')]
        .length>=20,{timeout:600000,polling:50}); expanded=true;}catch(e){}
  const tExpand=Date.now()-tc;
  await sleep(2500);
  const d=await p.evaluate(()=>{const v=document.querySelector('.explorer-folders-view');
    const rows=v.querySelector('.monaco-list-rows');
    return {renderedRows:v.querySelectorAll('.monaco-list-row').length,
            explorerNodes:v.querySelectorAll('*').length,
            docNodes:document.querySelectorAll('*').length,
            virtualHeightPx:rows?parseFloat(rows.style.height)||null:null,
            setSize:v.querySelector('.monaco-list-row')?.getAttribute('aria-setsize')};});
  const scroll=await p.evaluate(async()=>{
    const el=document.querySelector('.explorer-folders-view .monaco-scrollable-element');
    const bx=el.getBoundingClientRect(),lat=[];
    for(let i=0;i<25;i++){const t=performance.now();
      el.dispatchEvent(new WheelEvent('wheel',{deltaY:2500,bubbles:true,cancelable:true,clientX:bx.x+bx.width/2,clientY:bx.y+bx.height/2}));
      await new Promise(r=>requestAnimationFrame(()=>requestAnimationFrame(r)));lat.push(performance.now()-t);}
    lat.sort((a,b)=>a-b);return{medianMs:+lat[12].toFixed(1),p90Ms:+lat[22].toFixed(1),maxMs:+lat[24].toFixed(1)};});
  const after=await p.evaluate(()=>({rowsAfterScroll:document.querySelectorAll('.explorer-folders-view .monaco-list-row').length,
    docNodesAfterScroll:document.querySelectorAll('*').length}));
  const m=await p.metrics();
  await p.screenshot({path:`/var/tmp/yakscale/ide/shot-${label}.png`});
  console.log(JSON.stringify({label,tExplorer,tExpand,expanded,...d,...after,scroll,
    jsHeapMB:+(m.JSHeapUsedSize/1048576).toFixed(1),chromeNodes:m.Nodes}));
  await b.close();
})().catch(e=>{console.error('FAIL',e.message);process.exit(1)});
