# sbw — Spec del blocco 1: Nucleo
 
*space-between-worlds · spec 1.1 · 2026-10-25 · vocabolario: `glossario.md` 0.9*
 
Tutto ciò che è scritto qui è deciso. Le sole eccezioni sono i punti marcati **da verificare** (da confermare su una fonte al momento del codice) e le questioni aperte della sezione 8.
 
Questa spec è scritta per chi non ha alcun contesto. Il vocabolario di gioco (Mondo, Incarnazione, Veglia, Registro eventi, Elemento, Filo, Ambientazione, Leggi) è definito in `glossario.md` e qui si richiama in una riga. Tutto il resto si definisce nella sezione 3. Il registro eventi ha una spec propria: `spec-02-registro.md`.
 
---
 
## 1. Scopo e fase
 
**Scopo.** Il Nucleo è la parte di sbw che decide che cosa accade nel mondo quando qualcuno compie un'azione o passa del tempo. Riceve lo stato del mondo e una richiesta, e restituisce i fatti che ne derivano. Non comunica con nessuno, non scrive su disco, non legge l'orologio.
 
**Fase.** 1 (mondo locale, un giocatore). Le fasi successive ci costruiscono sopra senza riscriverlo (sezione 11).
 
**Blocchi e fasi citati.** sbw è diviso in blocchi; questa spec copre il blocco 1. Gli altri blocchi compaiono solo per dire che cosa sta fuori dal nucleo e chi se ne occupa; nessuno di essi fa parte di questa spec.
 
| Blocco | Cosa fa |
|---|---|
| 2 — Registro eventi | conserva gli eventi, fonte di verità del mondo (`spec-02-registro.md`) |
| 3 — Runtime | il processo sempre acceso che fa girare un mondo: riceve i comandi e li serializza, fornisce ora reale e materiale casuale, scrive nel registro, esegue gli effetti esterni |
| 4 — Accesso | come il giocatore si collega (SSH) e invia testo |
| 5 — Narratore | produce il testo per il giocatore tramite un LLM, leggendo stato ed eventi |
| 6 — Federazione | scambio di eventi tra mondi diversi |
| 8 — Formato dell'ambientazione | il formato in cui l'autore scrive ambientazione, stato iniziale e regole |
| 9 — Regolamento di riferimento | il regolamento distribuito di default con sbw, scritto con lo stesso meccanismo delle regole dell'autore |
| 13 — Identità del giocatore | struttura degli ID di giocatori e incarnazioni |
 
| Fase | Contenuto |
|---|---|
| 1 | mondo locale, un giocatore |
| 1a | mondo locale, più giocatori contemporanei nella stessa veglia |
| 3 | federazione tra mondi |
 
---
 
## 2. Dentro e fuori
 
**Dentro il nucleo:**
- decidere: dato stato e comando, produrre eventi o un rifiuto;
- applicare: dato stato ed evento, produrre il nuovo stato;
- il vocabolario chiuso degli effetti;
- la forma dello stato;
- una vista dello stato in sola lettura;
- la valutazione dei generatori di fili e della significatività;
- l'interfaccia con cui legge le regole;
- l'interfaccia per la casualità (solo il materiale casuale ricevuto, non la sua fonte).
**Fuori dal nucleo, con chi se ne occupa:**
 
| Cosa | Chi |
|---|---|
| Leggere l'orologio, dare l'ora reale | Runtime (blocco 3) |
| Rete, SSH, ricevere il testo del giocatore | Accesso (blocco 4) |
| Scrivere gli eventi, assegnare la posizione, tenere il registro | Registro eventi (blocco 2) |
| Tradurre il testo digitato in un comando strutturato | non ancora assegnato; fuori dal nucleo |
| Chiamare l'LLM, produrre testi | Narratore (blocco 5) |
| Ordinare e serializzare i comandi concorrenti | Runtime (blocco 3) |
| Effetti esterni (messaggi ai giocatori, echi verso altri mondi) | Runtime (blocco 3), solo su eventi vivi |
| Struttura degli ID di giocatori e incarnazioni | Identità del giocatore (blocco 13) |
| Flusso pubblicato verso altri mondi | Federazione (blocco 6) |
 
---
 
## 3. Concetti
 
Cos'è un MUD, per chi parte da zero: un mondo testuale sempre acceso. Ci si collega, si scrive "prendi lanterna", il mondo risponde. Il giro completo è questo:
 
1. Il giocatore scrive "prendi lanterna" (Accesso, blocco 4).
2. Il runtime (blocco 3) lo trasforma in un **comando** strutturato e aggiunge ciò che il nucleo non può procurarsi da solo (l'ora reale, il materiale per i tiri di dado).
3. Il **nucleo** (blocco 1) riceve stato e comando e restituisce **eventi**.
4. Il runtime scrive gli eventi nel **registro** (blocco 2) e aggiorna lo stato.
5. Il giocatore vede il risultato (narrazione, blocco 5).
**Mondo** (glossario). Un'ambientazione in esecuzione, con il suo stato e la sua storia.
**Incarnazione** (glossario). La presenza di un personaggio in un mondo: è "chi" agisce.
**Veglia** (glossario). Il periodo in cui un'incarnazione è dentro un mondo, da quando entra a quando esce.
**Filo** (glossario). Una storia (una quest, una faida) che attraversa più eventi.
**Registro eventi** (glossario). La cronologia di tutto ciò che accade in un mondo, in una sola linea, a cui si aggiunge senza mai modificare né cancellare. È la fonte di verità.
**Elemento** (glossario). Qualsiasi cosa notevole definita dall'ambientazione, con un ID stabile.
 
**Nucleo.** Una funzione pura: dati gli stessi ingredienti dà sempre lo stesso risultato e non fa nient'altro. Niente rete, disco, orologio, LLM, casualità vera. Ingredienti: stato, comando, regole. Risultato: eventi, oppure un rifiuto. Ha due operazioni:
- **decidere** (stato + comando → eventi, o rifiuto);
- **applicare** (stato + evento → nuovo stato).
**Comando.** Una richiesta che qualcosa accada, rivolta al nucleo, che può essere rifiutata. È strutturato (identificatori, non testo libero): "l'incarnazione A prende l'elemento L", non "prendi lanterna". Non è un fatto storico e non sta nel registro.
 
**Evento.** Un fatto accaduto nel mondo, deciso dal nucleo e scritto nel registro. Non si modifica né si cancella. "Ada prende la lanterna" è un comando; "Ada ha preso la lanterna" è un evento. Ogni evento ha due parti: un **significato** e un **effetto** (5.3).
 
**Significato.** L'etichetta di ciò che è accaduto ("preso", "ferito", "filo aperto"), con vocabolario aperto, definito dalle regole dell'autore. Serve a cronache, narratore, significatività e generatori di fili. Non serve a ricostruire lo stato.
 
**Effetto.** La lista dei cambiamenti di stato che l'evento produce, scelti da un vocabolario chiuso e fisso del nucleo (5.4), ciascuno con il valore prima e il valore dopo. "Applicare" interpreta solo l'effetto.
 
**Stato.** La fotografia completa del mondo in un istante. Non è la fonte di verità: si ottiene applicando in ordine tutti gli eventi del registro, a partire dalla genesi (5.5). Si può buttare e ricostruire.
 
**Replay.** Ricostruire lo stato rigiocando il registro. Deve dare sempre lo stesso stato e gli stessi fili.
 
**Determinismo.** Stessi ingredienti, stesso risultato.
 
**Regola.** Un pezzo dichiarato nell'ambientazione o nel regolamento di riferimento che dice cosa succede quando un tipo di comando incontra un certo stato ("prendere: se l'oggetto è qui e non è inchiodato, l'incarnazione lo ha"). Il nucleo senza regole è un motore vuoto.
 
**Tempo: tre cose distinte.**
- **Ora reale.** L'orologio del mondo vero. La legge solo il runtime e arriva al nucleo come dato nel comando. Avanza sempre e può sbagliare (gli orologi vengono corretti).
- **Posizione nel registro.** Il numero d'ordine di ogni evento (1, 2, 3, ...), assegnato dal registro. Stabilisce prima e dopo nella storia. Per l'ordine ci si fida della posizione, non dell'ora reale.
- **Tempo di finzione.** Il tempo dentro il gioco ("anno 800, sera"). È un dato nello stato come gli altri; può scorrere più veloce o più lento, e una meccanica di gioco può cambiarlo, riavvolgimento compreso.
Le prime due non si toccano mai; solo la terza è materia di gioco. È ciò che rende il riavvolgimento compatibile con "una sola linea temporale".
 
**Scadenza.** Una voce nello stato: "al tempo di finzione T il nucleo va svegliato per valutare X". Serve perché il mondo vive anche senza giocatori. Si riferisce sempre al tempo di finzione, mai all'ora reale.
 
**Seme e tiro (5.9).** Un dado nel nucleo non può essere casuale davvero: si parte da un materiale casuale (il seme) da cui si ricava una sequenza che sembra casuale ma è sempre la stessa. Il seme lo porta il runtime nel comando, preso da una fonte casuale vera. L'esito di ogni tiro è registrato nell'evento.
 
**Esposto / non esposto** (glossario). Etichetta su ogni evento, fissata alla scrittura, che decide se l'evento potrà essere inviato ad altri mondi.
 
**Livelli delle regole.** Le regole dell'autore arrivano in due forme: **livello 1**, contenuto dichiarativo (Fase 1); **livello 2**, moduli sandboxed (WASM o script), più avanti. Entrambi implementano la stessa interfaccia (5.10). Non esiste codice nativo dell'autore.
 
---
 
## 4. Nomi nel codice
 
| Termine | Identificatore inglese |
|---|---|
| Nucleo | crate `sbw-kernel` |
| Comando | `Command` |
| Evento | `Event` |
| Significato | `Meaning` |
| Effetto | `Effect` |
| Stato | `State` |
| Regola | `Rule` |
| Scadenza | `Deadline` |
| Seme | `Seed` |
| Ora reale | `WallTime` |
| Posizione (nel registro) | `Position` |
| Tempo di finzione | `WorldTime` |
| decidere / applicare | `decide` / `apply` |
 
---
 
## 5. Interfacce
 
Qui si descrive cosa entra e cosa esce, non come è scritto il codice.
 
### 5.1 Le due operazioni
 
- **decidere**: ingressi: vista in sola lettura dello stato, comando, regole. Uscita: una lista di eventi (anche vuota), oppure un rifiuto.
- **applicare**: ingressi: stato, evento. Uscita: il nuovo stato. Usa **solo l'effetto** dell'evento, mai le regole dell'autore.
Un comando accettato produce eventi; l'applicazione degli eventi dà lo stato nuovo. Reazioni e generatori di fili attivati dalle nuove condizioni producono a loro volta eventi nella stessa risposta (ciclo della risposta, 5.13).
 
### 5.2 Comando
 
| Campo | Contenuto |
|---|---|
| Chi | un'incarnazione (ID opaco); oppure un curatore (editore o game master, che agisce solo via comandi); oppure il mondo stesso (passaggio del tempo) |
| Cosa | un tipo di azione ("prendere"). I tipi non sono una lista chiusa: li definiscono le regole |
| Su cosa | i parametri: ID di elementi, valori |
| Dati dal runtime | l'ora reale e il materiale per i tiri |
 
Gli ID del giocatore e dell'incarnazione sono **tipi opachi**: la loro struttura non è ancora definita (blocco 13).
 
### 5.3 Evento
 
| Campo | Contenuto |
|---|---|
| Significato | etichetta, vocabolario aperto (regole dell'autore) |
| Effetto | lista di cambiamenti di stato dal vocabolario chiuso, ciascuno con valore prima e dopo |
| Coinvolti | ID di incarnazioni ed elementi |
| Veglia | **zero o una**: un evento senza giocatori (il mostro che si sposta a mezzanotte) non ha veglia |
| Fili | zero o più |
| Ora reale | fornita dal runtime nel comando |
| Esiti dei tiri | registrati nell'evento |
| Esposto / non esposto | etichetta fissata alla scrittura |
 
La **posizione** nel registro non è un campo scelto dal nucleo: la assegna il registro all'aggiunta.
 
**Esposizione in Fase 1.** Il nucleo produce ogni evento come **non esposto** e l'interfaccia delle regole non offre alcun modo di cambiarlo. Chi deciderà l'etichetta è una questione aperta (sezione 8).
 
### 5.4 Vocabolario degli effetti
 
Il principio è deciso; la lista definitiva si fissa in implementazione. Candidati, ciascuno con valore prima e dopo dove ha senso:
- creare un elemento (tipo, attributi iniziali, contenitore);
- rimuovere un elemento (conservando tutto ciò che serve per ricrearlo);
- spostare un elemento (da un contenitore a un altro);
- impostare un attributo (di un elemento o di un'incarnazione: nome, prima, dopo);
- aprire o chiudere un filo;
- programmare o annullare una scadenza;
- impostare l'ancora del tempo di finzione (tempo di finzione, ora reale di riferimento, ritmo; prima e dopo; vedi 5.8);
- aprire o chiudere una veglia.
La lista si verifica sui criteri 6, 7 e 18 e sull'apertura e chiusura delle veglie. Il riavvolgimento non richiede primitive nuove: è "sposta l'elemento da Y a X" e "imposta l'ancora con tempo di finzione da 800 a 790", cioè valori prima e dopo scambiati.
 
Un cambiamento che non rientra nelle primitive richiede una nuova primitiva nel nucleo, quindi una nuova versione del framework. È il prezzo di avere il replay indipendente dalle regole dell'autore. Varianti di termini (sinonimi) fanno capo a una sola primitiva.
 
### 5.5 Stato
 
Due strati:
 
- **Scheletro del nucleo**, uguale per tutti i mondi: elementi con ID, tipo e un solo contenitore; incarnazioni con ID e luogo; veglie aperte; fili aperti; scadenze; ancora del tempo di finzione.
- **Attributi**: coppie nome-valore agganciate a un elemento o a un'incarnazione. Il nome e il tipo li dichiara il regolamento (punti ferita, rabbia, "inchiodato"); il nucleo li conserva e li modifica senza capirne il significato.
**Struttura: albero a contenitore unico.** Ogni elemento sta in un solo posto alla volta (mondo → luogo → oggetto → oggetto contenuto).
 
**Tipi dei valori degli attributi:** interi, testo, booleani, riferimenti a ID. Niente virgola mobile. Motivo: il determinismo tra piattaforme con i float in Rust è ottenibile solo a condizioni precise (piattaforme IEEE 754 rigorose, niente funzioni trascendenti dipendenti dalla piattaforma come `sin`; vedi la guida al determinismo di Rapier). È fattibile ma fragile, e per un MUD non serve. Conseguenza: il regolamento di riferimento (d20) lavora con interi.
 
**Ordine di iterazione.** La documentazione di `std::collections::HashMap` dice che l'iterazione avviene "in arbitrary order" e che l'hasher di default è seminato a caso a ogni avvio. Quindi nel nucleo nessun risultato di `decide` o `apply` dipende dall'ordine di iterazione di una struttura non ordinata: si usano strutture ordinate (ad esempio `BTreeMap`) oppure si ordina prima di iterare.
 
**Stato iniziale: genesi.** Le prime voci del registro sono eventi di creazione generati dall'ambientazione (significato "genesi", effetti "creare un elemento" e "impostare l'ancora"). Il registro basta da solo: rigiocarlo non richiede la versione originale dell'ambientazione, coerente con l'invariante 3.
 
Gli stati dell'incarnazione utili al narratore (ferita, arrabbiata, sotto incantesimo) sono attributi come gli altri.
 
### 5.6 Vista dello stato in sola lettura
 
Chi legge lo stato senza poterlo modificare: le regole, chi traduce il testo in comando, il narratore. Criterio: ogni informazione che il narratore può usare è leggibile dallo stato o dagli eventi; non ha altri canali.
 
### 5.7 Rifiuto
 
- **Rifiuto ≠ fallimento.** Il *rifiuto* è un comando senza senso nello stato attuale (la lanterna è inchiodata, la spada non è qui): il mondo non fa nulla. Il *fallimento* è un comando valido che non riesce (il tiro per scassinare va male, il grimaldello si rompe): è un'**esecuzione** con significato "fallito" ed eventuali effetti, e va nel registro come ogni evento. Anche l'ostacolo attivo (la guardia che ti ferma) è una reazione "al posto di" che produce eventi, non un rifiuto.
- **Il rifiuto non si registra.** Zero eventi, stato invariato. Il nucleo restituisce al runtime il rifiuto con un **motivo strutturato** (codice più parametri: "non-presente", lanterna); il runtime lo comunica al giocatore.
- **Tentativi che contano** ("dopo tre tentativi la porta sigillata si accorge di te"): l'autore li modella come azione con effetto, non come rifiuto.
- Se servono i rifiuti per diagnostica o anti-abuso, li registra il runtime fuori dal registro, senza toccare il nucleo.
### 5.8 Tempo
 
- Il tempo entra come comando: il runtime manda periodicamente un comando di tempo con l'ora reale; chi lo invia è "il mondo stesso".
- Il nucleo tiene nello stato un indice delle scadenze ordinato per tempo di finzione e, a ogni comando di tempo, guarda solo le scadenze maturate. Nessuna scansione del mondo intero.
- Un comando di tempo che non produce effetti non produce eventi e non entra nel registro.
- Le scadenze si riferiscono al **tempo di finzione**. L'ora reale nell'evento è solo cronaca e non decide nulla. Così una meccanica che sposta il tempo di finzione (riavvolgimento, salto in avanti) tiene coerenti le scadenze.
- **Ancora.** Il tempo di finzione avanza senza un evento a ogni comando di tempo: nello stato c'è un'ancora, cioè un tempo di finzione, l'ora reale a cui vale e un **ritmo** (quanto tempo di finzione per unità di ora reale, espresso come rapporto tra due interi; zero = fermo). Il tempo di finzione corrente si calcola dall'ancora e dall'ora reale del comando. Un evento serve solo quando l'ancora cambia: pausa, cambio di ritmo, riavvolgimento, salto. L'effetto corrispondente è "impostare l'ancora" (prima e dopo).
- Il **ritmo iniziale** è un dato dell'ambientazione (blocco 8) e entra nello stato con la genesi; il ritmo corrente sta nello stato, perché le meccaniche possono cambiarlo e perché un cambio del ritmo nell'ambientazione non deve riscrivere i tempi di finzione passati in replay (invariante 3). Un mondo che non tocca mai il tempo ha ritmo corrente uguale a quello iniziale per sempre e zero eventi sul tempo.
- Cosa succede alle scadenze quando il tempo di finzione torna indietro (una scadenza già maturata torna futura; una non maturata si allontana) lo decidono le regole, che possono annullarle o riprogrammarle con effetti normali. Il nucleo non fa nulla di automatico.
### 5.9 Casualità
 
- Il materiale dei tiri lo porta il runtime nel comando: byte casuali presi dal sistema operativo. Il nucleo non ha casualità propria e nello stato non c'è nessun seme.
- Il nucleo espande il materiale in tiri con un generatore pseudo-casuale deterministico, seminato col materiale del comando. Ogni esito finisce nell'evento (con la ragione del tiro: "attacco, 1d20 → 14").
- Il replay non ritira mai: applica gli esiti registrati. Quindi un comando di tempo scartato che ha tirato non sposta nulla, e l'algoritmo del generatore non deve restare identico tra versioni del framework. Basta il determinismo di `decide` a parità di ingressi (criterio 1).
- Il "seme" della garanzia dei generatori di fili ("stesso stato e stesso seme, stesso filo") è il materiale del comando.
- Generatore: un PRNG nominato e documentato come portabile (famiglia ChaCha, crate `rand_chacha`), non `StdRng`, che la documentazione di `rand` dichiara non portabile tra piattaforme e versioni. Serve a non rompere i test a materiale fisso a ogni aggiornamento. **Da verificare:** la portabilità di `rand_chacha` sulla documentazione del crate.
- Riprodurre un bug richiede il comando col suo materiale, che il registro non conserva. Dimostrare ad altri mondi che i tiri non sono truccati è fuori da questa spec; il nucleo continua comunque a ricevere il materiale nel comando.
### 5.10 Interfaccia per le regole
 
Vincoli: le regole producono significato ed effetti dal vocabolario chiuso; in Fase 1 solo livello 1, con il regolamento di riferimento scritto con lo stesso meccanismo delle regole dell'autore; l'interfaccia deve già reggere il livello 2 senza riscriverla. La sua validità si conferma in implementazione.
 
**Forma esterna.** Il nucleo segue il pattern Decider (`decide` / `evolve` / stato iniziale). Differenza: `apply` (l'`evolve` del Decider) è fisso nel nucleo e lavora solo sugli effetti; le regole dell'autore entrano solo in `decide`.
 
**Le quattro funzioni**, per ogni tipo di azione:
 
| Funzione | Ingressi | Uscita | Riferimento |
|---|---|---|---|
| **disponibili** | vista dello stato, chi | comandi possibili per quel chi in quello stato | `legal` di GDL |
| **verifica** | vista, comando | ok, oppure rifiuto con motivo | Check di Inform 7 |
| **esecuzione** | vista, comando, tiri | significato + effetti | Carry out di Inform 7 |
| **reazioni** | vista, comando o eventi appena prodotti | eventi aggiuntivi nella stessa risposta | Before / Instead / After di Inform 7 |
 
- Le reazioni sono regole agganciate ad *altri* elementi (la trappola che scatta quando qualcuno prende la lanterna); possono scattare prima dell'azione, al suo posto o dopo.
- Il testo per il giocatore (il Report di Inform 7) non sta nel nucleo: lo fa il narratore leggendo stato ed eventi.
- **Percezione:** concetto del nucleo, non delle regole (il `sees` di GDL-II). Serve a **disponibili**, a chi traduce il testo in comando e al narratore: un oggetto che l'incarnazione non percepisce non compare tra i suoi comandi disponibili. Come l'autore dichiara cosa percepisce chi lo decide il blocco 8.
- **Catene di reazioni:** ordine di valutazione deterministico e limite di profondità, condiviso con i generatori di fili (5.13); il superamento del limite è un errore segnalato, non un troncamento silenzioso.
- **Contratto:** l'interfaccia si scrive in WIT (Component Model di WebAssembly) già in Fase 1; la implementeranno l'interprete del formato dichiarativo (livello 1) e i moduli WASM degli autori (livello 2). Il formato concreto delle regole dichiarative lo decide il blocco 8.
### 5.11 Generatori di fili e significatività
 
Un generatore è dichiarato nell'ambientazione (modello di filo più condizioni sullo stato). Garanzia: stesso stato e stesso seme, stesso filo.
 
- **Cos'è.** Una reazione che invece di agganciarsi a un'azione guarda una condizione sullo stato; il suo effetto è aprire o chiudere un filo. Anche la chiusura è una condizione dichiarata ("si chiude quando la lanterna torna nella Cantina o viene distrutta").
- **Cosa riceve.** Lo stato dopo l'applicazione degli eventi e gli eventi appena prodotti. Si valutano solo i generatori che osservano elementi toccati da quegli eventi (gli **elementi notevoli**): niente scansione del mondo intero.
- **Cosa produce.** Eventi di apertura e chiusura del filo, nella stessa risposta.
- **Appartenenza degli eventi ai fili.** Ogni filo aperto tiene l'elenco degli elementi e delle incarnazioni che segue. Un evento che ne coinvolge uno riceve quel filo nel campo "fili", assegnato dal nucleo quando produce l'evento; l'evento che ha fatto aprire il filo ci appartiene. Il campo è scritto nell'evento: in replay non si ricalcola nulla.
- **Significatività.** Nessun giudizio: soglie, condizioni ed elementi notevoli dichiarati dall'autore. Il nucleo valuta condizioni, non decide cosa è interessante.
- **Conseguenze.** Le decide l'autore, agganciando reazioni all'apertura o alla chiusura del filo; avvengono nella stessa risposta (5.13).
- I fili **riconosciuti** a posteriori da un editore passano da un comando del curatore, come ogni altro comando.
### 5.12 Ordine e concorrenza (vincolo)
 
Il runtime serializza i comandi. Il nucleo non sa che esista la concorrenza e resta puro.
 
### 5.13 Ciclo della risposta
 
Una risposta del nucleo a un comando è un ciclo:
 
1. **verifica**; se rifiuta, il ciclo finisce (5.7);
2. **esecuzione** (più le reazioni "prima" e "al posto di") produce i primi eventi;
3. gli eventi si applicano a una copia di lavoro dello stato;
4. sui nuovi eventi si valutano reazioni "dopo" e generatori di fili, che possono produrre altri eventi;
5. si torna al punto 3 finché non escono eventi nuovi, oppure si supera il limite di profondità.
Superato il limite: errore segnalato, nessun evento in uscita, stato invariato. Un solo meccanismo e un solo limite per reazioni e generatori. L'ordine di valutazione all'interno di ogni passo è deterministico (fissato dalle regole e, a parità, dagli ID).
 
Gli eventi di una risposta formano un gruppo che il registro scrive tutto o niente (`spec-02-registro.md`, 5.1).
 
---
 
## 6. Invarianti
 
1. Il nucleo è puro: nessun accesso a rete, disco, orologio, LLM, casualità di sistema.
2. Determinismo: stessi stato, comando, regole, seme, stesso risultato.
3. Il replay usa solo gli effetti degli eventi, mai le regole dell'autore: cambiare le regole non altera il passato.
4. Il registro è una sola linea senza rami. Il vocabolario degli effetti non contiene primitive per creare o scegliere linee temporali.
5. L'ora reale entra solo come dato nel comando.
6. Nulla di ciò che produce un modello (LLM, classificatore, altro) entra nello stato o nel registro. Un modello influenza il mondo solo in due modi: un umano accetta un suo output come comando, oppure traduce l'input di un umano in un comando scelto tra quelli disponibili. Non inventa comandi, e il comando passa comunque da decidere.
7. Gli effetti esterni (messaggi, echi, chiamate all'LLM) si eseguono solo su eventi vivi, mai rigiocando dal registro (invariante del runtime).
8. Ogni informazione che il narratore può usare è leggibile dallo stato o dagli eventi.
9. Il valore "prima" di un effetto coincide con lo stato corrente al momento dell'applicazione; una discordanza è un errore segnalato, non una modifica silenziosa.
10. Gli attributi hanno tipi senza virgola mobile.
11. Nessun risultato di `decide` o `apply` dipende dall'ordine di iterazione di una struttura non ordinata.
---
 
## 7. Decisioni già prese
 
- Un mondo ha una sola linea temporale. Una biforcazione è un atto di progettazione (la derivazione di un'ambientazione da un'altra), mai un evento di gioco.
- Il nucleo è una macchina a stati deterministica e pura.
- Il registro contiene solo eventi autosufficienti: ogni evento dice abbastanza per aggiornare lo stato senza rieseguire le regole. I comandi non stanno nel registro.
- Eventi con significato (vocabolario aperto) ed effetto (vocabolario chiuso, con valore prima e dopo).
- Stato = scheletro fisso più attributi dichiarati dalle regole; albero a contenitore unico.
- L'LLM non scrive stato; la descrizione di colore è output rigenerabile e può essere non deterministica, pre-generata e variata, ma l'informazione che contiene deve venire dallo stato.
- Estensione dell'autore: livello 1 (dichiarativo) in Fase 1, livello 2 (moduli sandboxed) più avanti. Nessun codice nativo d'autore.
- Nel registro non esistono eventi di controllo tecnico (es. hash dello stato ogni N eventi): l'hash dello stato sta negli snapshot (`spec-02-registro.md`, 5.5 e 5.6).
- Il nucleo è il crate `sbw-kernel`, nel workspace Cargo di sbw. sbw è distribuito come binario unico precompilato.
---
 
## 8. Questioni aperte che toccano il blocco
 
- **Chi assegna l'etichetta esposto / non esposto.** In Fase 1 la risposta è fissata (5.3): tutto non esposto, nessun modo di cambiarlo.
- **Chi traduce il testo in comando.** Fuori dal nucleo e non ancora assegnato. Il nucleo offre già ciò che serve a chiunque lo faccia: **disponibili** (5.10) e la vista dello stato (5.6).
Regola per l'agente: su una questione aperta, **ci si ferma e si chiede**; non si decide in silenzio.
 
---
 
## 9. Dipendenze
 
- **Blocco 2 (registro eventi):** riceve gli eventi, assegna la posizione, tiene la linea. Il nucleo non lo conosce.
- **Blocco 3 (runtime):** dà l'ora reale e il materiale per i tiri, serializza i comandi, esegue gli effetti esterni.
- **Blocco 8 (formato dell'ambientazione):** fornisce la genesi e le regole.
- **Blocco 9 (regolamento di riferimento):** modulo sopra il nucleo che usa l'interfaccia delle regole.
- **Blocco 13 (identità del giocatore):** struttura degli ID di giocatore e incarnazione; nel nucleo restano opachi.
---
 
## 10. Criteri di accettazione
 
Forma: **condizione di partenza, azione, esito osservabile**. I test con asserzioni si scrivono col codice, non qui. La numerazione è stabile.
 
1. **Determinismo di decidere.** Dato lo stesso stato, lo stesso comando (ora reale e materiale per i tiri inclusi) e le stesse regole, due esecuzioni producono la stessa sequenza di eventi.
2. **Replay.** Dato lo stato iniziale e un registro di eventi, applicarli in ordine produce lo stesso stato a ogni esecuzione e su ogni piattaforma supportata.
3. **Indipendenza dalle regole in replay.** Dato un registro e due versioni diverse delle regole, il replay produce lo stesso stato.
4. **Purezza.** Il nucleo, nella sua compilazione, non dipende da nulla che acceda a rete, file system, orologio o casualità di sistema.
5. **Coerenza di "prima".** Dato un evento il cui valore "prima" non coincide con lo stato corrente, applicarlo produce un errore segnalato e non modifica lo stato.
6. **Caso ordinario.** Dato un mondo con Ada nella Cantina e una lanterna notevole nella Cantina, e il comando "Ada prende la lanterna", il nucleo produce l'evento di significato "preso" con effetto "sposta la lanterna da Cantina a inventario di Ada"; se un generatore è dichiarato per quella lanterna, produce anche l'evento di apertura del filo; applicati allo stato, la lanterna è nell'inventario di Ada e il filo è aperto.
7. **Caso del riavvolgimento.** Dato lo stato del caso 6 e un comando di riavvolgimento che riporta la lanterna dove era prima (dall'inventario di Ada alla Cantina), il nucleo produce eventi nuovi con valori prima e dopo scambiati; il registro non perde né modifica nessun evento già scritto; lo stato finale ha la lanterna nella Cantina.
8. **Una sola linea.** Il vocabolario degli effetti non contiene nessuna primitiva per creare, scegliere o fondere linee temporali.
9. **Narratore.** Per ogni informazione che il narratore può usare per descrivere una scena, esiste un modo di leggerla dallo stato o dagli eventi.
10. **Tempo senza eventi.** Dato uno stato senza scadenze maturate al tempo di finzione corrente, un comando di tempo produce zero eventi.
11. **Scadenza maturata.** Dato uno stato con una scadenza al tempo di finzione T e un comando di tempo la cui ora reale porta il tempo di finzione oltre T, il nucleo produce gli eventi previsti dalla regola della scadenza e la scadenza esce dall'indice.
12. **Scadenze e riavvolgimento.** Dato uno stato con una scadenza al tempo di finzione T, un evento che riporta il tempo di finzione prima di T e poi comandi di tempo che non lo riportano oltre T, la scadenza non matura.
13. **Tiri riproducibili a parità di materiale.** Dato lo stesso stato, lo stesso comando con lo stesso materiale per i tiri e le stesse regole, gli esiti dei tiri registrati negli eventi sono identici; cambiando solo il materiale, possono cambiare.
14. **Replay senza tiri.** Dato un registro con eventi che contengono esiti di tiri, il replay produce lo stesso stato senza che il generatore pseudo-casuale venga mai invocato.
15. **Comandi disponibili.** Dato il caso 6 (Ada e la lanterna nella Cantina), **disponibili** per Ada include "prendere la lanterna"; dato lo stesso stato con la lanterna in un altro luogo, non la include.
16. **Reazione.** Dato il caso 6 con una trappola dichiarata che reagisce a "prendere la lanterna", il comando produce nella stessa risposta l'evento "preso" e l'evento della trappola, nell'ordine fissato.
17. **Catena di reazioni oltre il limite.** Date due reazioni che si innescano a vicenda senza fine, il comando termina con un errore segnalato, nessun evento viene prodotto e lo stato non cambia.
18. **Conseguenze dell'apertura di un filo nella stessa risposta.** Dato il caso 6 con un generatore per la lanterna e una reazione dell'autore all'apertura di quel filo ("si spengono le luci della Cantina"), il comando "Ada prende la lanterna" produce in una sola risposta: "preso", apertura del filo, spegnimento delle luci, in quest'ordine; tutti e tre portano il filo nel campo "fili".
19. **Solo generatori interessati.** Dato un mondo con un generatore che osserva la lanterna e un comando che non tocca la lanterna, quel generatore non viene valutato.
20. **Replay dei fili.** Dato un registro con eventi assegnati a fili, il replay produce gli stessi fili aperti e chiusi senza valutare nessun generatore.
21. **Rifiuto.** Dato il caso 6 con la lanterna inchiodata, il comando "Ada prende la lanterna" produce un rifiuto con motivo strutturato, zero eventi, e lo stato resta identico.
22. **Fallimento.** Dato un regolamento in cui scassinare richiede un tiro, un comando di scasso con un tiro fallito produce un evento di significato "fallito" con l'esito del tiro registrato, e passa dal registro come ogni evento.
---
 
## 11. Estensione futura
 
- **Livello 2 (moduli sandboxed):** compongono le primitive di effetto esistenti, non ne creano. L'interfaccia per le regole (5.10) è scritta perché lo reggano.
- **Fase 1a (più giocatori):** nessun cambiamento al nucleo; il runtime serializza i comandi.
- **Fase 3 (federazione):** gli eventi inviati ad altri mondi sono quelli marcati come esposti, etichetta già presente in ogni evento dalla Fase 1. Il campo "Chi" del comando si estenderà con "un altro mondo"; nessuna modifica ora.
- **Nuovi tipi di effetto:** nuova versione del framework.
---
 
## Fonti
 
- Fowler, Event Sourcing (2005): https://martinfowler.com/eaaDev/EventSourcing.html
- Appunti sul talk di Fowler (fonte secondaria): https://hackmd.io/@pierodibello/The-Many-Meanings-of-Event-Driven-Architecture
- arc42, Event Sourcing (fonte secondaria): https://quality.arc42.org/approaches/event-sourcing
- The Rust Rand Book, Reproducibility (StdRng non portabile): https://rust-random.github.io/book/crate-reprod.html
- Chassaing, Functional Event Sourcing Decider (2021): https://thinkbeforecoding.com/post/2021/12/17/functional-event-sourcing-decider
- Game Description Language (GDL, GDL-II): https://en.wikipedia.org/wiki/Game_Description_Language
- Inform 7 Handbook, Action Processing: https://inform-7-handbook.readthedocs.io/en/v2.0/chapter_4_actions/action_processing__summary/
- wasmtime, component model: https://docs.wasmtime.dev/api/wasmtime/component/index.html
- Rust std, HashMap (ordine di iterazione arbitrario, hasher seminato a caso): https://doc.rust-lang.org/std/collections/struct.HashMap.html
- Rapier, Determinism (condizioni per il determinismo dei float tra piattaforme): https://rapier.rs/docs/user_guides/templates/determinism/