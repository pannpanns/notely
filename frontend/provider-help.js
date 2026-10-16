/* Only these public provider pages may be opened by the desktop bridge. */
const providerHelp={
 gemini:{name:'Google AI Studio',url:'https://aistudio.google.com/apikey',steps:['Masuk dengan akun Google ke Google AI Studio.','Pilih Create API key, lalu pilih atau buat project Google Cloud.','Salin key, tempelkan di kolom API key Notely, lalu Simpan dan Tes respons.']},
 openai:{name:'OpenAI Platform',url:'https://platform.openai.com/api-keys',steps:['Masuk ke OpenAI Platform dan pilih project Anda.','Buat secret key pada halaman API keys.','Salin key ke Notely, lalu Simpan dan Tes respons.']},
 claude:{name:'Claude Console',url:'https://platform.claude.com/settings/keys',steps:['Masuk ke Claude Console dan buka API keys.','Pilih Create key, beri nama dan atur akun atau workspace.','Salin key ke Notely, lalu Simpan dan Tes respons.']},
 groq:{name:'GroqCloud',url:'https://console.groq.com/keys',steps:['Masuk ke GroqCloud.','Buka API Keys dan buat key baru.','Salin key ke Notely, lalu Simpan dan Tes respons.']},
 openrouter:{name:'OpenRouter',url:'https://openrouter.ai/settings/keys',steps:['Masuk ke OpenRouter dan buka Settings → API Keys.','Buat key dan atur batas pemakaian sesuai kebutuhan.','Salin key ke Notely, isi ID model, lalu Simpan dan Tes respons.']}
};
function providerKeyGuide(id){
 const help=providerHelp[id];
 if(!help)return '<div class="api-guide"><strong>API server sendiri</strong><p>Minta API key, endpoint kompatibel OpenAI, dan ID model dari pengelola server. Masukkan di bawah, lalu Simpan dan Tes respons.</p></div>';
 return `<details class="api-guide" open><summary>Cara mendapatkan API key ${esc(providers[id].name)}</summary><ol>${help.steps.map(step=>`<li>${esc(step)}</li>`).join('')}</ol><a class="button" href="${help.url}" data-provider-help="${id}" target="_blank" rel="noopener noreferrer">Buka ${help.name} ↗</a><p>AI online memerlukan API key. Model diproses di server penyedia, tanpa unduhan model. Kuota dan biaya mengikuti akun penyedia.</p></details>`;
}
document.addEventListener('click',async e=>{
 const link=e.target.closest('[data-provider-help]');if(!link||!native())return;
 e.preventDefault();
 try{await native()('open_provider_help',{provider:link.dataset.providerHelp});}catch(error){alertError(error);}
});
