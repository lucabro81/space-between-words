# sbw — Spec del blocco 2: Registro eventi
 
*space-between-worlds · spec 1.1 · 2026-10-25 · vocabolario: `glossario.md` 0.9 · nucleo: `spec-01-nucleo.md` 1.1*
 
Tutto ciò che è scritto qui è deciso. Le sole eccezioni sono i punti marcati **da verificare** (da confermare su una fonte al momento del codice) e le questioni aperte della sezione 8.
 
Questa spec è scritta per chi non ha alcun contesto. Il vocabolario di gioco è definito in `glossario.md`; il nucleo, che produce gli eventi, in `spec-01-nucleo.md`.
 
---
 
## 1. Scopo e fase
 
**Scopo.** Il registro eventi è la fonte di verità di un mondo: la cronologia append-only di tutto ciò che vi accade. Lo stato è una sua proiezione. Il registro riceve gli eventi dal runtime, assegna la posizione, li conserva e li restituisce.
 
**Fase.** 1 (mondo locale, un giocatore). Le fasi successive ci costruiscono sopra senza riscriverlo (sezione 11).
 
**Blocchi e fasi citati.** sbw è diviso in blocchi; questa spec copre il blocco 2. Gli altri blocchi compaiono solo per dire che cosa sta fuori dal registro e chi se ne occupa; nessuno di essi fa parte di questa spec.
 
| Blocco | Cosa fa |
|---|---|
| 1 — Nucleo | decide quali eventi produrre e li applica allo stato (`spec-01-nucleo.md`) |
| 3 — Runtime | il processo sempre acceso che fa girare un mondo: serializza i comandi, scrive nel registro, esegue gli effetti esterni |
| 6 — Federazione | scambio di eventi tra mondi diversi |
| 7 — Strumenti d'autore | la CLI `sbw`, che include gli strumenti di ispezione del registro (dump, verify, reindex) |
| 13 — Identità del giocatore | struttura degli ID di giocatori, incarnazioni e mondi |
 
| Fase | Contenuto |
|---|---|
| 1 | mondo locale, un giocatore |
| 1a | mondo locale, più giocatori contemporanei nella stessa veglia |
| 3 | federazione tra mondi |
 
---
 
## 2. Dentro e fuori
 
**Dentro il registro:**
- scrivere ogni risposta del nucleo come un'unica unità atomica (5.1);
- assegnare la posizione a ogni evento;
- conservare gli eventi in una sola linea, senza rami, senza modifiche né cancellazioni;
- conservare l'etichetta esposto / non esposto fissata alla scrittura (5.8);
- aprirsi con la genesi;
- garantire l'integrità con la catena di hash (5.5);
- servire letture per posizione, gruppo, veglia, filo, coinvolti ed eventi esposti (5.7, 5.8);
- tenere gli snapshot, in un file separato (5.6).
**Fuori dal registro, con chi se ne occupa:**
 
| Cosa | Chi |
|---|---|
| Decidere quali eventi produrre | Nucleo (blocco 1) |
| Applicare gli eventi allo stato | Nucleo (`apply`) |
| Serializzare i comandi, dare ora reale e materiale per i tiri | Runtime (blocco 3) |
| Eseguire gli effetti esterni (messaggi, echi, LLM) | Runtime (blocco 3), solo su eventi vivi |
| Decidere cosa è esposto | questione aperta (sezione 8) |
| Pubblicare gli eventi esposti verso altri mondi | Federazione (blocco 6) |
| Contenere comandi e rifiuti | **nessuno**: non stanno nel registro |
| Garantire la consegna degli effetti esterni | Runtime (blocco 3), dalla Fase 3 (5.9) |
 
---
 
## 3. Concetti
 
**Registro eventi** (glossario). Cronologia append-only, una sola linea senza rami, privata del mondo. Contiene solo eventi, mai comandi né rifiuti. Si apre con la genesi.
 
**Evento** (`spec-01-nucleo.md`, 5.3). Un fatto accaduto nel mondo, con significato, effetto e gli altri campi definiti dal nucleo. È autosufficiente: basta da solo per aggiornare lo stato senza rieseguire le regole.
 
**Posizione** (`spec-01-nucleo.md`). Numero d'ordine di ogni evento (1, 2, 3, ...), assegnato dal registro all'aggiunta. Stabilisce il prima e il dopo nella storia. Distinta dall'ora reale e dal tempo di finzione.
 
**Genesi** (glossario). Le prime voci del registro: eventi di creazione generati dall'ambientazione. Il registro basta da solo per il replay.
 
**Esposto / non esposto** (glossario). Etichetta di ogni evento, fissata alla scrittura; solo gli eventi esposti potranno essere inviati ad altri mondi.
 
**Risposta** (`spec-01-nucleo.md`, 5.13). L'insieme di eventi che il nucleo produce per un comando (es. "preso" + apertura filo + spegnimento luci). Nel registro è un **gruppo**: si scrive per intero o per niente (5.1).
 
**Replay** (`spec-01-nucleo.md`). Ricostruire lo stato rigiocando il registro con soli gli effetti; stesso stato e stessi fili a ogni esecuzione.
 
**Ancora del tempo di finzione** (`spec-01-nucleo.md`, 5.8). Tempo di finzione, ora reale di riferimento, ritmo; si registra solo quando cambia.
 
**Snapshot.** Stato materializzato a una certa posizione per non rigiocare tutto all'avvio; sempre buttabile e ricostruibile (5.6).
 
**Catena di hash.** Ogni record porta un hash che dipende da tutti i record precedenti: una modifica in un punto qualsiasi della storia diventa rilevabile (5.5).
 
---
 
## 4. Nomi nel codice
 
| Termine | Identificatore inglese |
|---|---|
| Blocco 2 | crate `sbw-eventlog` |
| Registro eventi | `EventLog` |
| Posizione | `Position` (`spec-01-nucleo.md`) |
| Evento | `Event` (`spec-01-nucleo.md`) |
| Colonne del record | `position`, `group_start`, `format_version`, `exposed`, `body`, `chain_hash` |
 
I nomi delle tabelle derivate (5.7) sono indicativi e si fissano al codice.
 
---
 
## 5. Interfacce
 
### 5.1 Unità di scrittura: la risposta
 
L'operazione di scrittura del registro è **aggiungere un gruppo di eventi**: tutti gli eventi che il nucleo ha prodotto per un comando (ciclo della risposta, `spec-01-nucleo.md` 5.13). Il gruppo si scrive per intero o per niente; dopo un crash, il registro contiene o tutti i suoi eventi o nessuno. Gli eventi del gruppo ricevono posizioni contigue, nell'ordine in cui il nucleo li ha prodotti.
 
Motivo: in replay i generatori di fili e le reazioni non si rivalutano (`spec-01-nucleo.md`, criterio 20). Se di "preso" + apertura filo + spegnimento luci restasse scritto solo "preso", il filo non si aprirebbe mai più: nessuna regola viene rieseguita per ricostruirlo. È lo stesso tutto-o-niente che il nucleo già applica quando supera il limite di profondità.
 
Una risposta vuota (zero eventi, es. comando di tempo senza effetti) non scrive nulla.
 
Il confine del gruppo resta visibile nei record (5.3, `group_start`).
 
### 5.2 Storage
 
Il registro è un database **SQLite**, in modalità **WAL** con **`synchronous=FULL`** impostato esplicitamente all'apertura (mai lasciato ai default di un wrapper). Ogni gruppo di eventi è una transazione.
 
- Con WAL e `synchronous=FULL` ogni commit sincronizza il WAL su disco: un gruppo confermato sopravvive a un crash del processo e a una perdita di corrente, se sistema operativo, filesystem e disco rispettano le loro garanzie di sincronizzazione. Con `NORMAL` un gruppo confermato può essere perso dopo una perdita di corrente: non ammesso.
- Il registro è accessibile solo tramite l'interfaccia del blocco 2, che non espone alcuna operazione di modifica o cancellazione di eventi. L'append-only lo garantisce il codice del blocco, non il motore.
- Il file sta nella cartella dati del mondo, separata dall'ambientazione: l'ambientazione è versionata e immutabile e non contiene stato.
**Scartati:** file append-only scritto a mano (crash recovery e indici da scrivere e testare da zero, nessun bisogno in Fase 1); redb (il suo README lo dichiara di qualità beta e poco diffuso in produzione).
 
**Se un giorno SQLite diventa un collo di bottiglia**, si sostituisce dentro il blocco 2: l'interfaccia non cambia e il lavoro da rifare è limitato a export e import dei record.
 
### 5.3 Forma di un record
 
**Corpo dell'evento: CBOR deterministico** (RFC 8949, sezione 4.2). Lo stesso evento produce sempre gli stessi byte: serve alla catena di hash (5.5) e, in Fase 3, alla firma, senza mai ricodificare la storia. Regole deterministiche principali: chiavi delle mappe ordinate per byte, lunghezze indefinite rifiutate. I float non compaiono (`spec-01-nucleo.md`, invariante 10), quindi le regole deterministiche sui float non si applicano. Per leggere il registro a occhio esiste un comando di dump in JSON (blocco 7): è uno strumento, non il formato.
 
**Scartati:** JSON canonico (JCS, RFC 8785: vincola i dati al sottoinsieme I-JSON; RFC informativo, fuori dal canale standard IETF); postcard / bincode (formato legato alle struct Rust, non standard, illeggibile da altri linguaggi: inadatto a eventi che altri mondi dovranno leggere).
 
**Colonne della tabella.** Fuori dal corpo solo ciò che il registro deve filtrare senza decodificare:
 
| Colonna | Contenuto |
|---|---|
| `position` | posizione, chiave primaria intera |
| `group_start` | posizione del primo evento della risposta a cui l'evento appartiene |
| `format_version` | versione del formato con cui il record è stato scritto |
| `exposed` | esposto / non esposto, copia del campo del corpo (5.8) |
| `body` | byte CBOR dell'evento |
| `chain_hash` | hash di catena (5.5) |
 
Posizione, `group_start`, `format_version` e `chain_hash` sono colonne del registro, non campi dell'evento prodotto dal nucleo. Gli indici per veglia, filo ed elemento sono tabelle derivate (5.7), fuori dalle colonne del record.
 
**Versione del formato.** Il registro non si migra mai riscrivendo i record. Ogni record porta la versione con cui è stato scritto; il codice di lettura sa leggere tutte le versioni passate e le converte in memoria alla forma corrente (tecnica nota come upcasting nell'event sourcing; conoscenza generale, non verificata su una fonte). Il codice di lettura delle versioni vecchie non si cancella.
 
**Identità di un evento.** Nessun UUID. Dentro il mondo l'evento si identifica con la sua posizione; fuori dal mondo con la coppia **(ID del mondo, posizione)**, unica per costruzione perché la linea è una sola. L'ID del mondo è opaco: la sua struttura non è ancora definita (blocco 13).
 
**Confine del gruppo.** Ogni evento porta `group_start`. Serve a chi legge cosa ha prodotto un comando, alla verifica che un gruppo sia completo e, in Fase 3, alla consegna degli effetti esterni (5.9).
 
### 5.4 Garanzie di scrittura
 
**Ordine (vincolo per il runtime, blocco 3).** `decide` → commit del gruppo nel registro → `apply` sullo stato in memoria → effetti esterni. Lo stato in memoria non cambia mai prima che il gruppo sia confermato.
- Crash dopo il commit e prima di `apply`: al riavvio il replay riallinea lo stato.
- Commit fallito (disco pieno, errore di I/O): stato invariato, il comando torna con un errore e può essere riprovato (il comando non è un fatto storico; ri-decidere con nuovo materiale casuale è ammesso).
- Scartato: `apply` prima del commit, perché produce uno stato in memoria che il registro non giustifica.
**Un solo scrittore.** Il processo del mondo, all'avvio, prende un lock esclusivo del sistema operativo su un file nella cartella dati del mondo e lo tiene per tutta la vita del processo. Un secondo processo che vuole scrivere sullo stesso mondo non parte. Gli strumenti (dump, ispezione, backup) aprono il database in sola lettura e funzionano anche con il mondo acceso: in WAL lettori e scrittore non si bloccano a vicenda. Il lock è consultivo: vale per il codice di `sbw`, che è l'unico ad accedere al registro.
- Scartato: `locking_mode=EXCLUSIVE` di SQLite, perché esclude anche i lettori esterni mentre il mondo è acceso.
- **Da verificare:** che il lock venga rilasciato dal sistema operativo alla morte del processo, senza lock residui dopo un crash (conoscenza generale).
**Controlli all'aggiunta (solo forma, mai significato).** Il registro rifiuta l'intero gruppo se:
- un corpo non è CBOR deterministico valido;
- la versione del formato non è quella corrente (si scrive solo nella versione corrente; le precedenti sono solo in lettura);
- manca l'etichetta esposto / non esposto, oppure la colonna `exposed` non coincide con il valore nel corpo.
Un gruppo vuoto non è un errore: è un'operazione nulla (criterio 4). Il significato degli eventi lo garantisce il nucleo, non il registro.
 
### 5.5 Integrità
 
**Catena di hash (Fase 1).** Ogni record porta `chain_hash(N) = H(chain_hash(N-1) ‖ position ‖ group_start ‖ format_version ‖ exposed ‖ body)`; per il primo evento della genesi, `chain_hash(N-1)` è un valore fisso dichiarato dal formato. Lo calcola il registro all'aggiunta. Una modifica a un qualsiasi byte della storia rompe tutte le catene successive.
- Protegge da corruzione del disco e da modifiche a mano del file. SQLite non mette checksum sulle pagine se non con l'estensione cksumvfs, che non si usa; la catena copre il contenuto del registro.
- Non protegge da chi riscrive l'intera catena: per quello servirà la firma (Fase 3).
- Verifica: uno strumento (`sbw log verify`, blocco 7) ricalcola la catena dalla genesi e segnala la prima posizione che non torna.
**Algoritmo: SHA-256**, perché è nello standard WebCrypto (verificabile nativamente da browser e da strumenti TypeScript); BLAKE3 non lo è. La velocità dell'hash è irrilevante per un MUD. L'algoritmo è fissato da `format_version`: se cambia, cambia la versione, e i record precedenti restano verificabili con l'algoritmo con cui sono stati scritti.
 
**La catena copre tutto il registro**, esposti e non esposti. Un'eventuale catena dei soli eventi esposti è materia della federazione e non modifica questa.
 
**Niente eventi di controllo nel registro.** Nessun evento con l'hash dello stato ogni N eventi: un controllo non è un fatto del mondo. Le divergenze di `apply` in replay le trova già l'invariante 9 del nucleo (il valore "prima" coincide con lo stato corrente) al primo effetto che tocca lo stato divergente; le altre le trova l'hash dello stato salvato con gli snapshot (5.6), confrontato in replay. Requisito: lo stato ha una codifica canonica.
 
**Firma (Fase 3, nulla da predisporre ora).** In Fase 3 si firmerà la testa della catena con la chiave del mondo: una firma copre tutta la storia fino a quella posizione. Chiave del mondo e regole di firma sono fuori da questa spec.
 
### 5.6 Snapshot
 
Uno snapshot è lo stato completo del mondo salvato a una posizione. All'avvio si carica e si rigiocano solo gli eventi successivi. È derivato: si può sempre buttare e ricostruire dal registro.
 
**Quando (Fase 1).** Alla chiusura pulita del processo del mondo e ogni N eventi, con N configurabile; il default si fissa misurando. Servono in Fase 1 soprattutto per il controllo di divergenza (5.5), non per le prestazioni.
 
**Dove.** In un file SQLite separato nella cartella dati del mondo, accanto al registro, mai nel database del registro. Cancellarlo è sempre sicuro: il mondo riparte dalla genesi. Scartato: tabella nel database del registro (mescola dati derivati e fonte di verità, gonfia i backup, introduce cancellazioni nel file degli eventi).
 
**Contenuto di uno snapshot:**
- posizione a cui vale;
- stato in CBOR deterministico (codifica canonica dello stato);
- hash SHA-256 dello stato;
- hash di catena del registro a quella posizione;
- versione del formato dello stato.
**Versioni: nessuna conversione.** Uno snapshot con versione del formato dello stato diversa da quella corrente si scarta; lo stato si ricostruisce dal registro.
 
**Avvio.** Si usa lo snapshot più recente valido: versione corrente e hash di catena uguale a quello del registro alla sua posizione. Se nessuno è valido, si riparte dalla genesi.
 
**Verifica completa** (`sbw log verify`, blocco 7). Replay dalla genesi; a ogni posizione con uno snapshot si confronta l'hash dello stato ricalcolato con quello salvato. Una differenza è un errore segnalato, mai corretto in silenzio.
 
**Scrittura.** Dopo commit e `apply`. Un fallimento nella scrittura di uno snapshot non ferma il mondo: si segnala nel log del processo e al prossimo avvio si rigioca di più.
 
### 5.7 Letture e indici
 
**Letture offerte dall'interfaccia (Fase 1):**
- per intervallo di posizioni;
- ultima posizione;
- per gruppo;
- per veglia;
- per filo;
- per elemento o incarnazione coinvolti.
La lettura degli eventi esposti è in 5.8. Il registro risponde solo a "cosa è successo": le domande sullo stato corrente passano dalla vista dello stato del nucleo.
 
**Indici: tabelle derivate nel database del registro, scritte nella stessa transazione del gruppo.** Veglia, fili e coinvolti sono già nel corpo dell'evento; gli indici li duplicano in forma interrogabile (colonna `vigil` e tabelle `event_threads`, `event_involved`; nomi indicativi). Sono atomici con il gruppo: mai indietro rispetto al registro, nessun cursore di avanzamento.
- La catena di hash copre solo le colonne del record, mai le tabelle derivate.
- Le tabelle derivate si possono svuotare e ricostruire dai corpi (`sbw log reindex`, blocco 7). È l'unica cancellazione ammessa nel file del registro e non tocca mai la tabella degli eventi.
- Un indice nuovo si popola rigiocando i corpi, senza migrare i record.
**Scartato:** indici in un file separato aggiornato dopo il commit (possono restare indietro rispetto al registro e richiedono cursore e recupero). Diverso dagli snapshot, che restano in un file separato perché grossi e periodici.
 
**Estensione futura:** ricerche più ricche (es. testo del significato) si aggiungono come altre tabelle derivate o proiezioni esterne, senza toccare il formato del record.
 
### 5.8 Esposto / non esposto
 
Chi decide l'etichetta è una questione aperta (sezione 8); in Fase 1 il nucleo produce ogni evento come non esposto (`spec-01-nucleo.md`, 5.3). Il registro fa solo questo:
- **Coerenza.** La colonna `exposed` è una copia del campo del corpo. All'aggiunta, se non coincidono, il gruppo è rifiutato (controlli di 5.4).
- **Default.** In assenza di regole dichiarate, un evento è **non esposto**; anche gli eventi di genesi. Motivo: l'etichetta non cambia mai, quindi un evento esposto per errore non si ritira, mentre un fatto non esposto si può sempre far circolare dopo con un evento nuovo.
- **Lettura.** "Eventi esposti a partire dalla posizione P" (posizione e corpo). Come vengono impacchettati e inviati ad altri mondi è fuori da questa spec (blocco 6).
### 5.9 Effetti esterni
 
Il registro non tiene traccia della consegna degli effetti esterni. In Fase 1 gli effetti esterni sono messaggi al giocatore locale e narrazione: se il processo muore tra il commit e l'invio, il messaggio è perso e il replay non lo rimanda (invariante 8); il giocatore, ricollegandosi, vede lo stato corretto. Nessuna garanzia di consegna in Fase 1.
 
In Fase 3 la consegna verso altri mondi richiederà un cursore per destinatario e lo scarto dei duplicati tramite (ID del mondo, posizione): materia del runtime e della federazione, non del registro.
 
### 5.10 Tempo
 
Nessun trattamento speciale. L'ancora del tempo di finzione cambia solo tramite un evento ordinario con effetto "impostare l'ancora" (`spec-01-nucleo.md`, 5.8); il tempo corrente si calcola, non si registra.
 
---
 
## 6. Invarianti
 
1. Una sola linea, senza rami.
2. Append-only: nessuna modifica né cancellazione di eventi scritti.
3. Contiene solo eventi.
4. La posizione la assegna il registro.
5. L'etichetta esposto / non esposto è fissata alla scrittura.
6. Il registro basta da solo per il replay (genesi).
7. Lo stato è una proiezione e si può buttare e ricostruire.
8. Gli effetti esterni si eseguono solo su eventi vivi, mai in replay (invariante del runtime).
9. Gli eventi di una risposta del nucleo sono scritti tutti o nessuno, con posizioni contigue.
10. Un gruppo confermato è durevole: sopravvive a crash del processo e perdita di corrente (nei limiti delle garanzie di sistema operativo, filesystem e disco).
11. Lo stesso evento si codifica sempre negli stessi byte.
12. Un record scritto non si riscrive mai, neanche per migrare il formato: ogni versione del formato resta leggibile per sempre.
13. Fuori dal mondo, un evento è identificato da (ID del mondo, posizione).
14. Un mondo ha un solo scrittore del registro alla volta.
15. Lo stato in memoria non riflette mai un gruppo non ancora confermato dal registro.
16. Ogni record porta un hash di catena che dipende da tutti i record precedenti.
17. Il registro contiene solo fatti del mondo: nessun record di controllo tecnico.
18. Gli snapshot sono derivati: cancellarli non cambia la storia né lo stato ricostruito.
19. Lo stato ha una codifica canonica: lo stesso stato produce sempre gli stessi byte.
20. Gli indici sono sempre allineati al registro (stessa transazione) e sempre ricostruibili dai corpi degli eventi.
---
 
## 7. Decisioni già prese
 
- Il registro contiene solo eventi autosufficienti: ogni evento basta per aggiornare lo stato senza rieseguire le regole.
- Un mondo ha una sola linea temporale; il riavvolgimento sono eventi nuovi in coda.
- Il tempo di finzione si registra solo quando l'ancora cambia.
- Stato iniziale da genesi registrata.
- Il registro è privato del mondo: non si condivide in blocco con altri mondi; solo gli eventi esposti potranno essere inviati.
- Linguaggio Rust; tutto locale, nessun servizio cloud di terzi.
---
 
## 8. Questioni aperte che toccano il blocco
 
- **Chi assegna l'etichetta esposto / non esposto.** In Fase 1 tutto è non esposto (5.8); il registro conserva e verifica l'etichetta, non la decide.
- **Firma del registro e catena dei soli esposti** (Fase 3). La catena di hash (5.5) è la base; nulla da predisporre ora.
Regola per l'agente: su una questione aperta, **ci si ferma e si chiede**.
 
---
 
## 9. Dipendenze
 
- **Blocco 1 (nucleo):** produce gli eventi; il registro non lo conosce.
- **Blocco 3 (runtime):** scrive nel registro, serializza i comandi, gestisce la consegna degli effetti esterni.
- **Blocco 6 (federazione):** leggerà gli eventi esposti.
- **Blocco 13 (identità):** ID di giocatore, incarnazione e mondo negli eventi; opachi finché la loro struttura non è definita.
---
 
## 10. Criteri di accettazione
 
Forma: **condizione di partenza, azione, esito osservabile**. La numerazione è stabile.
 
1. **Risposta atomica.** Dato un registro con ultima posizione N e una risposta di tre eventi ("preso", apertura filo, spegnimento luci), se il processo viene terminato durante la scrittura, al riavvio il registro ha ultima posizione N oppure N+3, mai N+1 o N+2.
2. **Posizioni contigue e ordinate.** Dato un gruppo di eventi aggiunto con successo dopo la posizione N, gli eventi hanno posizioni N+1, N+2, ... nell'ordine in cui il nucleo li ha prodotti.
3. **Durabilità.** Dato un gruppo di cui l'aggiunta è stata confermata, dopo una terminazione forzata del processo il gruppo è presente nel registro alla riapertura.
4. **Risposta vuota.** Data una risposta senza eventi, aggiungerla non cambia l'ultima posizione del registro.
5. **Nessuna modifica.** L'interfaccia del registro non offre alcuna operazione che modifichi o cancelli un evento già scritto.
6. **Configurazione esplicita.** All'apertura del registro, la modalità del journal risulta WAL e `synchronous` risulta FULL, indipendentemente dai default della libreria usata.
7. **Codifica deterministica.** Dato lo stesso evento, due codifiche (anche in processi diversi o con le chiavi delle mappe inserite in ordine diverso) producono byte identici.
8. **Codifica non deterministica rifiutata.** Dato un corpo che viola le regole del CBOR deterministico (es. chiavi non ordinate o lunghezza indefinita), la lettura lo segnala come errore e non lo interpreta.
9. **Versioni vecchie leggibili.** Dato un registro con record scritti in una versione precedente del formato, la lettura restituisce gli eventi nella forma corrente e i record su disco restano byte per byte invariati.
10. **Confine del gruppo.** Dato un gruppo di tre eventi aggiunto dopo la posizione N, tutti e tre hanno `group_start` = N+1.
11. **Un solo scrittore.** Dato un mondo con il processo scrittore acceso, l'avvio di un secondo scrittore sullo stesso mondo fallisce con un errore esplicito e il registro resta invariato.
12. **Lettura con il mondo acceso.** Dato un mondo con il processo scrittore acceso, uno strumento in sola lettura legge il registro fino all'ultimo gruppo confermato.
13. **Lock dopo un crash.** Dato un processo scrittore terminato in modo forzato, un nuovo scrittore sullo stesso mondo parte senza intervento manuale.
14. **Gruppo malformato.** Dato un gruppo di tre eventi in cui il secondo ha un corpo non valido, l'aggiunta è rifiutata e l'ultima posizione del registro non cambia.
15. **Commit fallito.** Dato un registro che non riesce a scrivere (es. disco pieno), un comando produce un errore e lo stato del mondo resta identico a prima del comando.
16. **Catena integra.** Dato un registro scritto solo dal blocco 2, la verifica della catena dalla genesi all'ultima posizione non segnala errori.
17. **Manomissione rilevata.** Dato un registro in cui un byte del corpo dell'evento in posizione K è stato modificato fuori da `sbw`, la verifica segnala la posizione K come prima posizione non valida.
18. **Avvio da snapshot.** Dato un registro con ultima posizione N e uno snapshot valido alla posizione S < N, l'avvio produce lo stesso stato del replay dalla genesi.
19. **Snapshot cancellati.** Dato un mondo a cui è stato tolto il file degli snapshot, l'avvio riparte dalla genesi e produce lo stesso stato.
20. **Snapshot di versione vecchia.** Dato uno snapshot con versione del formato dello stato diversa da quella corrente, l'avvio lo ignora e usa uno snapshot precedente valido o la genesi.
21. **Snapshot estraneo.** Dato uno snapshot il cui hash di catena non coincide con quello del registro alla sua posizione, l'avvio lo ignora.
22. **Divergenza rilevata.** Dato uno snapshot il cui hash dello stato non coincide con lo stato ricalcolato in replay a quella posizione, la verifica completa segnala la posizione.
23. **Lettura per filo.** Dato il caso della lanterna (tre eventi con il filo F nel campo "fili") e altri eventi senza F, la lettura per filo F restituisce esattamente quei tre eventi, in ordine di posizione.
24. **Indici atomici.** Dato un gruppo il cui commit fallisce, nessuna tabella derivata contiene righe di quel gruppo.
25. **Ricostruzione degli indici.** Date tabelle derivate svuotate e ricostruite dai corpi, ogni lettura indicizzata restituisce gli stessi risultati di prima, e la verifica della catena di hash non cambia esito.
26. **Esposto incoerente.** Dato un gruppo in cui un evento ha la colonna `exposed` diversa dal valore nel corpo, l'aggiunta è rifiutata.
27. **Default non esposto.** Dato un comando senza regole che si pronuncino sull'esposizione, gli eventi prodotti sono non esposti e la lettura degli esposti non li restituisce.
---
 
## 11. Estensione futura
 
- **Fase 1a (più giocatori):** nessun cambiamento; il runtime serializza.
- **Fase 3 (federazione):** gli eventi inviati ad altri mondi sono quelli già marcati come esposti (5.8); la firma si applica alla testa della catena di hash (5.5); la consegna usa un cursore per destinatario nel runtime (5.9). Nulla del registro va riscritto.
- **Crescita e ciclo di vita** (segmenti, archiviazione, compattazione): non servono in Fase 1; si affrontano quando le dimensioni diventano un problema misurato.
---
 
## Fonti
 
- SQLite, pragma synchronous: https://www.sqlite.org/pragma.html#pragma_synchronous
- SQLite, write-ahead logging: https://www.sqlite.org/wal.html
- SQLite, atomic commit: https://www.sqlite.org/atomiccommit.html
- Andrew Ayer, "SQLite's Durability Settings are a Mess" (WAL + FULL è durevole; i wrapper possono cambiare il default): https://www.agwa.name/blog/post/sqlite_durability
- redb, README (formato stabile, qualità beta): https://github.com/cberner/redb/
- RFC 8949, CBOR, sezione 4.2 (codifica deterministica): https://www.rfc-editor.org/rfc/rfc8949#section-4.2 (**da verificare** sul testo primario prima del codice; il riassunto delle regole è verificato tramite il draft CBOR Core: https://www.ietf.org/archive/id/draft-rundgren-cbor-core-06.html)
- RFC 8785, JSON Canonicalization Scheme: https://www.rfc-editor.org/info/rfc8785
- SQLite, The Checksum VFS Shim: https://www.sqlite.org/cksumvfs.html
- Deno std/crypto (BLAKE3 fuori dallo standard WebCrypto, SHA-256 dentro; fonte secondaria): https://deno.land/std@0.224.0/crypto/mod.ts
**Da verificare** al codice (conoscenza generale): crate Rust per SQLite (`rusqlite` con SQLite compilato nel binario); comportamento di fsync sui filesystem supportati.