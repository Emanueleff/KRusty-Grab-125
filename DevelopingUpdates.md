
# Ema 
## hot keys:
esempio con egui / eframes

https://github.com/tauri-apps/global-hotkey/blob/dev/examples/egui.rs

funziona ma lo prende solo se sei all'interno dell'applicazione. Potrebbe anche starci bene così 

Se sei al di fuori dell'applicazione: 
- si mangia la lettera che hai scritto 
- quando riattivi la scheda dell'applicazione esegue la funzione voluta

Possibili soluzioni -> forse mettendo la receive da un altra parte e non nell'update (che viene chiamato ogni volta che viene ri renderizzata la window), si potrebbe ottenere qualcosa. 
Problema: voglio davvero disabilitare permanentemente dei tasti?

