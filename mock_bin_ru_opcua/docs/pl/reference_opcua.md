# Referencja OPC UA — przestrzeń adresowa (RU/OPC UA)

*🌍 [FR](../fr/reference_opcua.md) · [EN](../en/reference_opcua.md) · [DE](../de/reference_opcua.md) · [ES](../es/reference_opcua.md) · [IT](../it/reference_opcua.md) · [PT](../pt/reference_opcua.md) · [NL](../nl/reference_opcua.md) · **PL***

> Źródło prawdy: [`opcua_server.rs`](../../src/opcua_server.rs) (deklaracja węzłów
> + wywołania zwrotne). Każda zmiana tablicy następuje **w tym pliku** i jest tu
> odzwierciedlana.

---

## 1. Endpoint i bezpieczeństwo

URL to `opc.tcp://<bind_ip>:<port>/` (domyślnie `opc.tcp://0.0.0.0:4840/`),
transport OPC UA TCP binarny. **Bezpieczeństwo** jest konfigurowalne (sekcja
`[security]` pliku TOML / modal *Ustawienia*) i wyznacza udostępniany endpoint:

| Tryb | `encryption` | Polityka | Tryb bezpieczeństwa | Tokeny |
|---|:--:|---|---|---|
| **Nieszyfrowany** (domyślnie) | `false` | `None` | `None` | `Anonymous` |
| **Szyfrowany** | `true` | `Basic256Sha256` | `SignAndEncrypt` | `Anonymous` (jeśli `allow_anonymous`) i/lub użytkownik/hasło |

- **Nieszyfrowany**: ani uwierzytelniania, ani szyfrowania. Udostępniać wyłącznie
  w **sieci zaufanej**. Natychmiastowy start (brak certyfikatu).
- **Szyfrowany**: **samopodpisany certyfikat instancji** jest generowany przy
  pierwszym uruchomieniu (w `pki/`). Zaufanie do certyfikatów klientów jest
  **konfigurowalne** (`trust_client_certs`): **automatyczne** (domyślnie, wygodne
  dla symulatora) lub **ścisłe** — klient musi wtedy być wcześniej zatwierdzony
  w `pki/trusted/` (w przeciwnym razie trafia do `pki/rejected/` i jest odrzucany).
  Uwierzytelnianie za pomocą **użytkownika/hasła**, jeśli `username` jest podane;
  w przeciwnym razie (lub dodatkowo) token **anonimowy**, jeśli `allow_anonymous`.
  ⚠️ Generowanie RSA może zająć kilka sekund przy pierwszym uruchomieniu (debug).

Ustawienia (`[security]`): `encryption` (bool), `allow_anonymous` (bool),
`trust_client_certs` (bool, domyślnie `true`), `username` (puste = brak
uwierzytelniania hasłem), `password` (jawnym tekstem — **tylko symulator**).

---

## 2. Namespace

| Indeks | URI |
|---|---|
| `0` | `http://opcfoundation.org/UA/` (rdzeniowy namespace OPC UA) |
| `ns` | `urn:cesam-lab:ru-opcua` (namespace aplikacyjny) |

Indeks `ns` namespace'u aplikacyjnego jest przydzielany dynamicznie przy starcie;
klient rozwiązuje go przez `IN GetNamespaceArray` / usługę *Browse*. Poniższe węzły
biznesowe tam żyją.

---

## 3. Węzły (pod folderem `Objects`)

Każdy węzeł jest typu `Variable`; jego `NodeId` ma postać `ns=<ns>;s=<nazwa>`.

| BrowseName | NodeId (`s=`) | Typ | Dostęp | Wielkość |
|---|---|---|:--:|---|
| `Setpoint` | `Setpoint` | `Double` | R/W | Wartość zadana (jednostka procesu) |
| `ProcessValue` | `ProcessValue` | `Double` | R | Pomiar (PV) |
| `Output` | `Output` | `Double` | R | Wyjście sterujące (%) |
| `ManualOutput` | `ManualOutput` | `Double` | R/W | Wyjście narzucone w trybie ręcznym (%) |
| `Run` | `Run` | `Boolean` | R/W | Praca / zatrzymanie regulacji |
| `Auto` | `Auto` | `Boolean` | R/W | Tryb automatyczny (PID) vs ręczny |

- **Odczyty**: obsługiwane przez wywołanie zwrotne odczytujące **współdzielony
  zrzut**; są więc „żywe” i **próbkowalne** przez subskrypcje (*Subscription* /
  *MonitoredItem*).
- **Zapisy**: kierowane do aktora symulacji. Wartości są **odkażane** (nieskończone
  odrzucane, wartość zadana ograniczona, wyjście ręczne ograniczone do `[0, 100]`).

---

## 4. Mapowanie na stan biznesowy

| Węzeł | Skutek zapisu | Źródło odczytu |
|---|---|---|
| `Setpoint` | `Command::SetSetpoint` (ograniczona `[sp_min, sp_max]`) | `snapshot.setpoint` |
| `ManualOutput` | `Command::SetManualOutput` (ograniczona `[0, 100]`) | `snapshot.manual_output` |
| `Run` | `Command::SetRun` | `snapshot.run` |
| `Auto` | `Command::SetAuto` | `snapshot.auto` |
| `ProcessValue` | — (tylko odczyt) | `snapshot.pv` |
| `Output` | — (tylko odczyt) | `snapshot.output` |

Zapis nieoczekiwanego typu zwraca `Bad_TypeMismatch`; zapis bez wartości —
`Bad_NothingToDo`. Typ `Float` jest akceptowany obok `Double` dla węzłów
numerycznych.

---

## 5. Przykłady (klient OPC UA)

Za pomocą ogólnego klienta (UaExpert, `opcua` CLI itp.) połącz się z
`opc.tcp://127.0.0.1:4840/`, bezpieczeństwo **None**, użytkownik **Anonymous**,
następnie:

```text
# Odczyt pomiaru i wartości zadanej
Read  ns=<ns>;s=ProcessValue   → 60.0
Read  ns=<ns>;s=Setpoint       → 60.0

# Uruchomienie + nowa wartość zadana
Write ns=<ns>;s=Run        = true
Write ns=<ns>;s=Setpoint   = 80.0

# Przełączenie w tryb ręczny i wyjście narzucone na 40 %
Write ns=<ns>;s=Auto         = false
Write ns=<ns>;s=ManualOutput = 40.0
```

Subskrypcja (*Subscribe* / *MonitoredItem*) do `ProcessValue` i `Output` pozwala
śledzić dynamikę procesu w czasie rzeczywistym.
