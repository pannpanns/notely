/* Animate navigation only, never polling updates or text entry. */
let lastNavigationView='';
function animateNavigation(){
 const view=[state.page,state.page==='settings'?state.tab:'',state.page==='notes'?state.selected:state.page==='recordings'?state.libraryFolder:''].join(':');
 if(view===lastNavigationView)return;
 lastNavigationView=view;
 if(window.matchMedia?.('(prefers-reduced-motion: reduce)').matches)return;
 const page=document.getElementById('main-page');
 const surface=state.page==='settings'?page?.querySelector('.settings-body'):page?.firstElementChild;
 surface?.animate?.([{opacity:0,transform:'translateY(12px) scale(.992)'},{opacity:1,transform:'translateY(0) scale(1)'}],{duration:320,easing:'cubic-bezier(.22,1,.36,1)'});
 const cards=surface?.querySelectorAll('.home-choice,.bento-panel,.folder-card,.library-recording,.setting-card,.recent-note,.upcoming-mini-item,.local-model,.model-card,.deadline-card');
 [...(cards||[])].slice(0,10).forEach((card,i)=>card.animate?.([{opacity:0,transform:'translateY(12px) scale(.99)'},{opacity:1,transform:'translateY(0) scale(1)'}],{duration:300,delay:i*32,easing:'cubic-bezier(.22,1,.36,1)',fill:'backwards'}));
}
const beforeMotionRender=render;
render=function(){beforeMotionRender();animateNavigation();};
const beforeMotionRefresh=refreshPage;
refreshPage=function(){beforeMotionRefresh();animateNavigation();};
animateNavigation();
