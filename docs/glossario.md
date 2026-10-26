# sbw — Glossario

*space-between-worlds · bozza 0.9 · 2026-10-25*

Glossario dei concetti del framework. Qui ci sono i nomi usati in documentazione, CLI e nei confronti di chi crea mondi. I nomi di classi e interfacce nel codice (inglese, tecnici) non stanno nel glossario e si decidono quando servono.

**Regola dei nomi:** i nomi aulici si usano solo per concetti nuovi, propri del sistema; dove il concetto esiste già nel linguaggio comune si usa il nome comune.

Le questioni non ancora decise sono in `questioni-aperte.md`. Il dettaglio tecnico del nucleo (comandi, eventi, tempo, casualità, regole) è in `spec-01-nucleo.md`; quello del registro eventi (storage, atomicità, integrità, snapshot) è in `spec-02-registro.md`.

Stato di ogni voce:
- **deciso**: concordato, si cambia solo con buone ragioni
- **proposto**: definizione di lavoro, da validare

---

## Principi di fondo

1. **sbw è il framework con cui si costruiscono e si fanno girare MUD.** Non è un MUD e non li genera da solo. Come un framework applicativo (es. Nuxt): un comando fa lo scaffolding, il framework mette a disposizione runtime, API e strumenti, da lì in poi il contenuto è lavoro dell'autore. *(deciso)*
2. **Protocollo nel framework, servizio sopra.** Il framework contiene il protocollo di rete: formato del contratto, forma dell'essenza, identificatori, eventi tra mondi. Relay, discovery, hosting, modelli di lore e curatela condivisa sono un servizio costruito sopra, self-hostabile o offerto a pagamento. Due mondi devono potersi federare senza passare da un servizio centrale. *(deciso)*
3. **Le Leggi sono pubbliche.** Un protocollo aperto con leggi non pubbliche non è aperto. *(deciso)*
4. **Livelli con velocità di cambiamento diverse.** Più un concetto è in alto, più è stabile e minimo. *(deciso)*
5. **Il determinismo decide, l'LLM narra.** Stato, regole, nascita dei fili e significatività degli eventi sono decisi da logica deterministica dichiarata. L'LLM produce testo e può proporre, mai decidere. Un modello (LLM, classificatore, altro) può anche tradurre l'input di un giocatore in un [comando](#comando) scelto tra quelli disponibili: non ne inventa, non scrive stato, e il comando passa comunque dalle regole. *(deciso)*

| Livello | Cambia | Chi lo decide |
|---|---|---|
| [Universo](#universo) / [Leggi](#leggi) | quasi mai | il framework |
| [Federazione](#federazione) / [Contratto](#contratto) | raramente, a versioni | i mondi che vi aderiscono |
| [Ambientazione](#ambientazione) | a versioni | chi la scrive |
| [Mondo](#mondo) | continuamente | chi lo gestisce e chi ci gioca |

---

## Strati: framework, contenuto, stato

**Stato:** deciso

| Strato | Cos'è | Dove vive | Chi lo fa |
|---|---|---|---|
| **Framework (sbw)** | Runtime (comandi, veglie, [registro eventi](#registro-eventi)), strumenti di authoring, protocollo di rete | Il codice di sbw | Chi sviluppa sbw |
| **[Ambientazione](#ambientazione)** | Il contenuto: elementi con ID, lore di base, estensioni del regolamento, mappature, generatori di fili | File versionati dell'autore | L'autore |
| **[Mondo](#mondo)** | Lo stato: registro eventi, storia, canon | Sull'istanza che lo ospita | Chi gioca, con la revisione degli editori |

Flusso tipico per un MUD da zero: `sbw new` crea un'ambientazione (scaffolding) → l'autore scrive elementi e lore di base → `sbw instantiate` crea un mondo dall'ambientazione → si gioca, le veglie producono eventi, i fili nascono e si chiudono, gli editori ne traggono cronache e il canon del mondo cresce. I nomi dei comandi sono indicativi.

---

## Termini

*In ordine alfabetico.*

### Ambientazione
**Stato:** deciso

Ciò che un mondo **è per definizione**: l'ambientazione in senso narrativo (geografia, lore di base, elementi notevoli) più le estensioni locali del regolamento, la mappatura dell'essenza verso le regole locali e i generatori di fili. È statica e versionata. Rapporto con il mondo: **classe → oggetto**.

La **lore di base** sta qui: è lore *progettata*, scritta dall'autore. Gli strumenti di sbw possono assisterlo (bozze generate dall'LLM, controlli di coerenza), ma ogni contenuto è approvato da un umano.

**Correlati:** [Mondo](#mondo) · [Elemento](#elemento) · [Derivazione](#derivazione) · [Regolamento di riferimento](#regolamento-di-riferimento) · [Filo](#filo) · [Canon](#canon) · [MUD](#mud) · [Genesi](#genesi)

### Canon
**Stato:** deciso (principio), proposto (dettaglio)

L'insieme degli eventi e della lore di un mondo riconosciuti come veri: è lore *accaduta*, e vale solo per quel mondo. La narrazione prodotta dall'IA è derivata e rigenerabile; diventa canon solo attraverso le cronache approvate da **editori umani**, che la incastrano in modo coerente con quanto esiste. Si canonizzano soprattutto fili, non singole veglie.

**Risalita.** Il canon di un mondo può diventare lore di base della sua ambientazione, ma solo con un atto esplicito dell'autore o degli editori che pubblicano una **nuova versione** dell'ambientazione (o una derivazione). Mai in automatico. *(deciso)*

Catena completa:

> **[Veglia](#veglia)** (il tempo vissuto) → **[Fili](#filo)** (le storie che lo attraversano) → **[Cronaca](#cronaca)** (il racconto curato) → **Canon** (ciò che è vero)

**Correlati:** [Cronaca](#cronaca) · [Filo](#filo) · [Mondo](#mondo) · [Ambientazione](#ambientazione) · [Derivazione](#derivazione) · [Contratto](#contratto)

### Comando
**Stato:** deciso

Una richiesta che qualcosa accada in un mondo, rivolta al nucleo, che può essere rifiutata. È strutturata (chi, cosa, su cosa), non testo libero: "l'incarnazione A prende l'elemento L", non "prendi lanterna". Chi lo emette: un'incarnazione, un curatore (editore, game master) o il mondo stesso (lo scorrere del tempo). Il testo scritto dal giocatore diventa comando attraverso un parser o un modello che sceglie tra i comandi disponibili. Un comando non è un fatto storico e non sta nel registro eventi: ci stanno gli [eventi](#evento) che produce.

**Correlati:** [Evento](#evento) · [Rifiuto](#rifiuto) · [Mondo](#mondo) · [Incarnazione](#incarnazione) · [Registro eventi](#registro-eventi)

### Contratto
**Stato:** proposto

Ciò che i mondi di una federazione accettano per farne parte: regolamento (con versione), regole di canonizzazione, regole sugli effetti tra incarnazioni. Va versionato fin dall'inizio.

**Correlati:** [Federazione](#federazione) · [Regolamento di riferimento](#regolamento-di-riferimento) · [Incarnazione](#incarnazione) · [Canon](#canon) · [Leggi](#leggi)

### Cronaca
**Stato:** proposto

Il racconto scritto e curato di ciò che è accaduto: prodotto dagli editori a partire da veglie e fili, con l'eventuale assistenza dell'LLM. È il passaggio intermedio tra ciò che è successo e ciò che diventa vero.

**Correlati:** [Veglia](#veglia) · [Filo](#filo) · [Canon](#canon)

### Derivazione
**Stato:** deciso (nome), proposto (dettaglio)

La derivazione di un'ambientazione da un'altra (un fork). L'ambientazione derivata **eredita gli ID** degli elementi che non modifica; gli elementi nuovi o riscritti ricevono ID nuovi. Ogni ambientazione registra da dove proviene. L'insieme delle derivazioni di un'ambientazione forma un **albero delle derivazioni**: i suoi antenati e i suoi discendenti.

Una derivazione è un atto di **progettazione**: non esistono biforcazioni prodotte dal gioco a runtime (vedi [Mondo](#mondo)).

**Correlati:** [Ambientazione](#ambientazione) · [Elemento](#elemento) · [Riflessi](#riflessi) · [Canon](#canon)

### Eco
**Stato:** proposto

Traccia lasciata da un giocatore (oggetto, messaggio, segno del passaggio) che diventa visibile ad altri in modo asincrono: nello stesso mondo in un momento successivo, o in un riflesso attraverso un varco.

**Correlati:** [Varco](#varco) · [Riflessi](#riflessi) · [Veglia](#veglia) · [Stato e visibilità](#stato-e-visibilità) · [Modalità di gioco](#modalità-di-gioco)

### Elemento
**Stato:** proposto

Qualsiasi cosa notevole definita in un'ambientazione (luogo, entità, oggetto) con un **ID stabile**. È la base su cui si definiscono parentela tra ambientazioni e varchi. Un elemento può essere marcato come notevole: ciò che gli accade è un evento significativo per i generatori di fili.

**Correlati:** [Ambientazione](#ambientazione) · [Derivazione](#derivazione) · [Riflessi](#riflessi) · [Varco](#varco) · [Filo](#filo) · [Leggi](#leggi)

### Essenza
**Stato:** proposto

Il personaggio nella sua **continuità narrativa**: identità, nome, origine, storia. Esiste a livello di universo ed è trasportabile ovunque.

**Correlati:** [Giocatore](#giocatore) · [Incarnazione](#incarnazione) · [Leggi](#leggi) · [Federazione](#federazione)

### Evento
**Stato:** deciso

Un fatto accaduto in un mondo, registrato nel [registro eventi](#registro-eventi). È **autosufficiente**: contiene tutto ciò che serve per aggiornare lo stato senza rieseguire le regole, così cambiare le regole non riscrive il passato. Ha due parti:

- **Significato**: cosa è successo nel linguaggio del mondo ("preso", "fallito", "inciso"). Vocabolario aperto, lo definiscono le regole dell'autore. È ciò che leggono narratore, cronache e altri mondi.
- **Effetto**: come cambia lo stato, espresso con un vocabolario chiuso e fisso di primitive del nucleo (creare, rimuovere, spostare un elemento, impostare un attributo, aprire o chiudere un filo, e simili), ciascuna con valore prima e dopo. Il vocabolario degli effetti è interno al framework e non fa parte delle [Leggi](#leggi).

Un evento porta anche: chi è coinvolto, zero o una [veglia](#veglia), zero o più [fili](#filo), l'ora reale, gli esiti dei tiri, l'etichetta esposto / non esposto. La posizione nel registro la assegna il registro.

**Correlati:** [Comando](#comando) · [Registro eventi](#registro-eventi) · [Filo](#filo) · [Veglia](#veglia) · [Flusso pubblicato](#flusso-pubblicato) · [Leggi](#leggi)

### Federazione
**Stato:** deciso (nome), proposto (dettaglio)

Un insieme di mondi legati da un **contratto** comune. Proposta di partenza: un mondo può appartenere a più federazioni; una federazione è piatta.

**Correlati:** [Contratto](#contratto) · [Mondo](#mondo) · [Regolamento di riferimento](#regolamento-di-riferimento) · [Universo](#universo)

### Filo
**Stato:** deciso (concetto e nome), proposto (dettaglio)

Una storia: una quest, una faida, una caccia, la vendetta della progenie di un vampiro. Un'incarnazione può vivere più fili contemporaneamente; un filo può attraversare molte veglie e coinvolgere più incarnazioni. Ogni evento appartiene a zero o una veglia (quando) e a zero o più fili (quale storia): sono due dimensioni ortogonali. Un evento senza giocatori (il mostro che si sposta a mezzanotte) non ha veglia.

Un filo nasce in tre modi, sempre senza giudizio di un LLM:
1. **Dichiarato**: scritto dall'autore nell'ambientazione, con apertura e chiusura esplicite.
2. **Generato**: un **generatore** dichiarato nell'ambientazione (modello di filo + condizioni sullo stato del mondo) si attiva quando le azioni dei giocatori portano il mondo in quelle condizioni. È deterministico: stesso stato e stesso seme, stesso filo. Le azioni plasmano il mondo, quindi non tutto è deciso a priori.
3. **Riconosciuto**: un editore, rileggendo le veglie, raggruppa a posteriori eventi in un filo che nessuna regola aveva previsto. L'LLM può proporre raggruppamenti, l'editore conferma.

L'appartenenza di un evento ai fili la assegna il nucleo quando produce l'evento, e resta scritta nell'evento: il replay non la ricalcola. *(deciso)*

"Significativo" non è un'opinione: lo definiscono soglie, condizioni ed elementi notevoli dichiarati dall'autore.

**Correlati:** [Veglia](#veglia) · [Ambientazione](#ambientazione) · [Elemento](#elemento) · [Evento](#evento) · [Cronaca](#cronaca) · [Canon](#canon) · [Leggi](#leggi)

### Flusso pubblicato
**Stato:** proposto

Il sottoinsieme degli eventi di un mondo marcati come esposti, che altri mondi possono ricevere. Ogni evento del flusso è attribuibile al solo mondo che lo ha pubblicato. È ciò che permette a un mondo di manifestare localmente fatti accaduti altrove (echi, passaggi dai varchi) senza che nessuno copi il registro eventi di un altro: un mondo che entra in una federazione non importa la storia degli altri mondi, solo ciò che sta nel flusso. Chi riceve decide localmente, con regole proprie, come manifestare ciò che riceve; ogni mondo resta l'unica sede autoritativa del proprio stato. Gli eventi sono marcati come esposti secondo regole dichiarate. I destinatari tipici sono i riflessi attraverso i varchi.

**Correlati:** [Registro eventi](#registro-eventi) · [Evento](#evento) · [Eco](#eco) · [Varco](#varco) · [Riflessi](#riflessi) · [Contratto](#contratto) · [Mondo](#mondo)

### Genesi
**Stato:** deciso

Le prime voci del registro eventi di un mondo: eventi di creazione generati dall'ambientazione quando il mondo nasce. Grazie alla genesi il registro basta da solo: rigiocarlo non richiede la versione originale dell'ambientazione.

**Correlati:** [Registro eventi](#registro-eventi) · [Ambientazione](#ambientazione) · [Mondo](#mondo) · [Evento](#evento)

### Giocatore
**Stato:** proposto

L'individuo dietro le essenze: è a lui che fanno riferimento essenze e incarnazioni. Ha un **ID globale e unico**, come ogni entità dell'universo, che non cambia quando un mondo entra o esce da una federazione.

L'ID deriva dalla **prima chiave pubblica** del giocatore, generata dal giocatore stesso sul proprio client e mai da sbw: il framework non custodisce segreti. La prima connessione con una chiave sconosciuta è il momento in cui nasce il Giocatore. L'ID non viene mai ricalcolato.

**Correlati:** [Essenza](#essenza) · [Incarnazione](#incarnazione) · [Leggi](#leggi) · [Mondo](#mondo)

### Incarnazione
**Stato:** proposto

La presenza di un'essenza in un mondo specifico, con le meccaniche di quel mondo. La **continuità meccanica** (statistiche, abilità, inventario) esiste solo tra mondi con regolamento compatibile. In un mondo incompatibile l'essenza dà vita a una nuova incarnazione con le regole locali, e le incarnazioni precedenti continuano a esistere dove sono. Le incarnazioni sono indipendenti tra loro salvo diversa regola del contratto.

**Correlati:** [Giocatore](#giocatore) · [Essenza](#essenza) · [Mondo](#mondo) · [Regolamento di riferimento](#regolamento-di-riferimento) · [Contratto](#contratto) · [Veglia](#veglia)

### Leggi
**Stato:** deciso (principio), proposto (contenuto)

La logica intima dell'universo: ciò senza cui due mondi non potrebbero nemmeno parlarsi. Comprende la specifica del framework e del protocollo. Contenuto previsto:
- identità globale e unica di ogni entità (mondi, ambientazioni, giocatori, essenze, elementi)
- forma minima di un **evento esposto** (significato, coinvolti, varco, ora reale) e del tempo. Il vocabolario degli effetti **non** ne fa parte: un mondo che riceve un evento ne legge il significato e lo manifesta con regole proprie, non applica mai gli effetti di un altro mondo al proprio stato
- **esiti dei tiri registrati negli eventi**: il materiale casuale lo fornisce il runtime, il replay applica gli esiti registrati e non ritira mai. L'algoritmo di casualità non è nelle Leggi
- forma minima e trasportabile di un'essenza
- meccanismo con cui un regolamento dichiara attributi, estensioni e mappature

**Non** contengono un regolamento né un insieme minimo di statistiche. Le Leggi devono restare minime: ogni regola qui vale per tutti i mondi per sempre.

**Correlati:** [Universo](#universo) · [Elemento](#elemento) · [Evento](#evento) · [Giocatore](#giocatore) · [Essenza](#essenza) · [Regolamento di riferimento](#regolamento-di-riferimento) · [Contratto](#contratto) · [Filo](#filo)

### Modalità di gioco

**Stato:** proposto. La modalità è definita da un solo parametro: **con chi condividi la sincronia**.

| Modalità | Sincronia | Gli altri esistono come |
|---|---|---|
| **Sincrona** (MUD classico) | tutto il mondo | presenze dirette |
| **Coop / ibrida** | la propria veglia condivisa | compagni dentro la veglia, echi fuori |
| **Asincrona** (AMUD, Async Multi User Dungeon) | solo il giocatore | solo echi |

La modalità asincrona è sceglibile, può essere esclusa, oppure essere il fulcro di un mondo (AMUD). Non è un requisito del core: un mondo isolato in single player deve sempre funzionare.

**Correlati:** [Veglia](#veglia) · [Eco](#eco) · [Stato e visibilità](#stato-e-visibilità) · [MUD](#mud)

### Mondo
**Stato:** deciso

Ciò che un'ambientazione **è fisicamente in un dato momento** in una data istanza: stato, storia, canon proprio. Ha un'unica sede autoritativa del proprio stato: il suo runtime, che applica i comandi e scrive il registro eventi. Più persone possono curarlo (autore, editori, game master), ma nessuna scrive lo stato direttamente: agiscono tramite comandi che il runtime valida e applica. L'identità di un mondo è il suo ID globale, non il suo nome: due mondi creati indipendentemente con lo stesso nome restano due mondi distinti. Esporre un mondo lo rende raggiungibile, non ne cambia l'identità. Federarsi non comporta mai la fusione di due mondi.

**Una sola linea temporale.** Un mondo ha una sola storia: il gioco non produce linee temporali parallele né biforcazioni a runtime. Ciò che nella finzione "riavvolge il tempo" è una meccanica di gioco, cioè comandi che producono nuovi eventi in coda al registro; il registro non si riscrive mai. Una biforcazione è un atto di progettazione (vedi [Derivazione](#derivazione)), non un evento di gioco. *(deciso)*

**Correlati:** [Ambientazione](#ambientazione) · [Federazione](#federazione) · [Riflessi](#riflessi) · [Incarnazione](#incarnazione) · [Veglia](#veglia) · [Registro eventi](#registro-eventi) · [Canon](#canon) · [Stato e visibilità](#stato-e-visibilità) · [Tempo di finzione](#tempo-di-finzione)

### MUD
**Stato:** deciso

Un'ambientazione più i mondi che ne nascono, in esecuzione sul runtime di sbw. Senza ambientazione il runtime è un motore vuoto; senza runtime l'ambientazione è solo documentazione. Un MUD la cui modalità è asincrona è un **AMUD** (vedi [Modalità di gioco](#modalità-di-gioco)).

**Correlati:** [Ambientazione](#ambientazione) · [Mondo](#mondo) · [Modalità di gioco](#modalità-di-gioco) · [Universo](#universo)

### Registro eventi
**Stato:** deciso

La cronologia append-only di tutto ciò che accade in un mondo: la fonte di verità. Lo stato del mondo ne è una proiezione. È una **sola linea, senza rami** (vedi [Mondo](#mondo)). È **privato del mondo**: non si condivide in blocco con altri mondi. Ogni evento è classificato fin dalla scrittura come **esposto** o **non esposto**: solo gli esposti entrano nel [flusso pubblicato](#flusso-pubblicato). In assenza di regole dichiarate un evento è non esposto; l'etichetta non cambia mai. Contiene solo [eventi](#evento), mai comandi né rifiuti; si apre con la [genesi](#genesi). Dettaglio tecnico in `spec-02-registro.md`.

**Correlati:** [Mondo](#mondo) · [Evento](#evento) · [Genesi](#genesi) · [Veglia](#veglia) · [Flusso pubblicato](#flusso-pubblicato) · [Stato e visibilità](#stato-e-visibilità) · [Leggi](#leggi)

### Regolamento di riferimento
**Stato:** proposto

Il regolamento distribuito di default con il framework (un d20 adattato con le regole derivate da ether). Ogni federazione lo usa salvo dichiarare altro; mondi e federazioni possono **estenderlo** in modo compatibile. Regolamenti incompatibili non sono vietati ma restano isolati: non si federano con chi usa il riferimento senza una mappatura esplicita. Le statistiche del personaggio sono definite qui, non nelle Leggi.

**Correlati:** [Contratto](#contratto) · [Federazione](#federazione) · [Leggi](#leggi) · [Incarnazione](#incarnazione) · [Ambientazione](#ambientazione)

### Rifiuto
**Stato:** deciso

La risposta del nucleo a un comando che non ha senso nello stato attuale (la lanterna è inchiodata, la spada non è qui): il mondo non fa nulla. Non produce eventi e non si registra; il giocatore riceve un motivo. Da non confondere con il **fallimento**: un comando valido che non riesce (il tiro per scassinare va male) è un'azione eseguita, produce eventi e finisce nel registro.

**Correlati:** [Comando](#comando) · [Evento](#evento) · [Registro eventi](#registro-eventi)

### Riflessi
**Stato:** proposto

Relazione tra mondi che condividono elementi con lo stesso ID:
- istanze della stessa ambientazione → condividono tutti gli elementi
- istanze di ambientazioni sulla stessa linea di derivazione → condividono gli elementi ereditati
- istanze di ambientazioni indipendenti → non condividono nulla, non sono riflessi

La parentela si misura elemento per elemento, non in blocco: non esiste una soglia di "quasi uguale".

**Correlati:** [Mondo](#mondo) · [Elemento](#elemento) · [Derivazione](#derivazione) · [Varco](#varco) · [Eco](#eco)

### Scadenza
**Stato:** deciso

Un appuntamento che un mondo dà a se stesso: "al tempo di finzione T valuta X" (il veleno che agisce tra tre ore di gioco, la porta che si richiude a mezzanotte). Serve perché il mondo vive anche senza giocatori. Si riferisce sempre al [tempo di finzione](#tempo-di-finzione), mai all'ora reale.

**Correlati:** [Tempo di finzione](#tempo-di-finzione) · [Mondo](#mondo) · [Evento](#evento)

### Stato e visibilità

**Stato:** deciso (principio), proposto (dettaglio)

Due cose distinte, con tempi distinti:

- **Scrittura dello stato**: sempre immediata. Un'azione cambia il mondo nel momento in cui avviene, per chiunque condivida la sincronia.
- **Visibilità verso l'esterno**: echi verso giocatori asincroni, echi verso i riflessi, materiale per le cronache. I suoi tempi dipendono dalla modalità: immediati, oppure alla chiusura della veglia.

Lo stato non è mai provvisorio; è la visibilità ad avere tempi diversi. Niente transazioni lunghe, niente conflitti alla chiusura.

**Correlati:** [Mondo](#mondo) · [Veglia](#veglia) · [Eco](#eco) · [Cronaca](#cronaca) · [Registro eventi](#registro-eventi) · [Modalità di gioco](#modalità-di-gioco)

### Tempo di finzione
**Stato:** deciso

Il tempo dentro il gioco ("anno 800, sera"), distinto dall'**ora reale** (l'orologio del mondo vero) e dalla **posizione** di un evento nel registro (l'ordine della storia). È l'unico dei tre che è materia di gioco: una meccanica può fermarlo, accelerarlo o riavvolgerlo, sempre con nuovi eventi in coda al registro.

Scorre secondo un'**ancora**: un tempo di finzione, l'ora reale a cui vale e un **ritmo** (quanto tempo di finzione per unità di ora reale, espresso come rapporto tra due interi; zero vuol dire che avanza solo quando lo fanno avanzare le regole). Il ritmo iniziale lo fissa l'ambientazione. Un evento serve solo quando l'ancora cambia: un mondo che scorre tranquillo non scrive nulla.

**Correlati:** [Scadenza](#scadenza) · [Mondo](#mondo) · [Registro eventi](#registro-eventi) · [Ambientazione](#ambientazione)

### Universo
**Stato:** deciso

L'insieme di tutto ciò che è costruito con sbw e delle regole che ne governano l'esistenza. Non è uno spazio né un'entità con comportamento: nessuno lo vede tutto e nessuno ne ha l'autorità. Esiste un solo universo.

**Correlati:** [Leggi](#leggi) · [Federazione](#federazione) · [MUD](#mud)

### Varco
**Stato:** proposto

Un elemento il cui ID è condiviso tra due o più riflessi, e quindi punto in cui i mondi si toccano. Un mondo dichiara quali elementi espone come varchi.

**Correlati:** [Elemento](#elemento) · [Riflessi](#riflessi) · [Eco](#eco) · [Mondo](#mondo)

### Veglia
**Stato:** deciso

Un frammento di esistenza di un'incarnazione in un mondo: tutto il tempo passato a fare cose, da quando vi entra a quando ne esce intenzionalmente. Non implica che qualcosa venga registrato come storia. Bazzicare senza meta è una veglia. Le disconnessioni accidentali non la chiudono: rientrando entro una finestra di tolleranza la veglia prosegue. Un'incarnazione vive una veglia alla volta; più giocatori possono condividere la stessa veglia (coop).

La **connessione** è un concetto solo tecnico (trasporto) e non fa parte del glossario di gioco.

**Correlati:** [Incarnazione](#incarnazione) · [Mondo](#mondo) · [Filo](#filo) · [Cronaca](#cronaca) · [Eco](#eco) · [Stato e visibilità](#stato-e-visibilità) · [Modalità di gioco](#modalità-di-gioco)
