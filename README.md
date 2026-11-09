<div align="center">

<img src="docs/images/logo.svg" alt="Devrem logosu" width="96" height="96" />

# Devrem

**Modern, hızlı ve ücretsiz elektronik devre simülatörü**

Devreni çiz, çalıştır, akımın aktığını gör.

**🇹🇷 Türkçe** · [🇬🇧 English](README.en.md)

[![Lisans: MIT](https://img.shields.io/badge/Lisans-MIT-22C55E?style=flat-square)](LICENSE)
![Platform](https://img.shields.io/badge/Windows-10%20%2F%2011-0078D4?style=flat-square&logo=windows&logoColor=white)
![Rust](https://img.shields.io/badge/Motor-Rust-B7410E?style=flat-square&logo=rust&logoColor=white)
![Tauri](https://img.shields.io/badge/Tauri-2-24C8DB?style=flat-square&logo=tauri&logoColor=white)
![React](https://img.shields.io/badge/React-19-61DAFB?style=flat-square&logo=react&logoColor=black)
![Dil](https://img.shields.io/badge/Aray%C3%BCz-T%C3%BCrk%C3%A7e%20%7C%20English-F5A524?style=flat-square)

[**📥 İndir**](../../releases/latest) ·
[Özellikler](#-öne-çıkanlar) ·
[Hızlı Başlangıç](#-hızlı-başlangıç) ·
[Kaynaktan Derleme](#-kaynaktan-derleme) ·
[Mimari](docs/ARCHITECTURE.md) ·
[Katkıda Bulun](#-katkıda-bulunma)

<br />

<img src="docs/images/demo-555.gif" alt="555 zamanlayıcıyla yanıp sönen LED: teller gerilime göre renkleniyor, akım hareketli noktalarla akıyor" width="720" />

<sub>555 zamanlayıcıyla yanıp sönen LED — gerçek zamanlı simülasyon</sub>

</div>

---

## Devrem nedir?

Devrem; analog ve dijital devreleri **gerçek zamanlı** simüle eden bir masaüstü
uygulamasıdır. Simülasyon motoru sıfırdan **Rust** ile yazıldı; arayüz koyu
temalı, sade ve Türkçe.

Bir değeri değiştirdiğinde sonucu anında görürsün: teller gerilime göre
renklenir, akım hareketli noktalarla akar, LED'ler parlar, osiloskop dalga
biçimlerini çizer.

|                    |                                                              |
| ------------------ | ------------------------------------------------------------ |
| 🎓 **Öğrenciler**  | Ders konularını görerek ve deneyerek öğrenmek                |
| 🔧 **Hobiciler**   | Devreyi kurmadan önce sanal ortamda denemek                  |
| 👩‍🏫 **Eğitmenler**  | Derste canlı devre gösterimi, öğrencilerle `.dvrm` paylaşımı |
| 👨‍🔬 **Mühendisler** | Hızlı fikir denemesi, AC analizi, DC tarama, SPICE çıktısı   |

**Neden Devrem?**

- 🆓 **Ücretsiz, hesapsız, açık kaynak** — kayıt, abonelik veya reklam yok; MIT lisanslı.
- ⚡ **Gerçek zamanlı** — simülasyon çalışırken anahtarlara tıkla, potansiyometreyi kaydır; devrenin durumu korunur.
- 🧠 **Doğrulanmış motor** — MNA tabanlı çözücü, analitik sonuçlarla karşılaştıran 81 testle sınanıyor.
- 🔀 **Analog ve dijital bir arada** — transistörler, op-amp ve 555 ile lojik kapılar ve flip-flop'lar aynı devrede çalışır.
- 🪶 **Hafif** — ~4 MB kurulum dosyası, çevrimdışı çalışır.

---

## ✨ Öne Çıkanlar

<table>
  <tr>
    <td width="50%" valign="top">
      <h3>🔀 Analog + dijital</h3>
      <img src="docs/images/demo-hex.gif" alt="Saat üreteci, dört T flip-flop, onaltılık çözücü ve 7 segment gösterge ile 0'dan F'ye sayan devre" />
      <p>Saat üreteci, flip-flop'lar, onaltılık çözücü ve 7 segment gösterge ile 0'dan F'ye sayan sayıcı.</p>
    </td>
    <td width="50%" valign="top">
      <h3>📈 Canlı osiloskop</h3>
      <img src="docs/images/oscilloscope.png" alt="Eviren op-amp yükseltecinin giriş ve çıkış dalga biçimleri osiloskopta" />
      <p>Voltmetre, ampermetre ve lojik prob okumaları simülasyon sırasında canlı çizilir.</p>
    </td>
  </tr>
  <tr>
    <td width="50%" valign="top">
      <h3>〰️ AC analizi (Bode)</h3>
      <img src="docs/images/ac-analysis.png" alt="RC alçak geçiren filtrenin Bode diyagramı" />
      <p>Frekans tepkisi: kazanç (dB) ve faz (°). RC filtrede 1 kHz'de −3 dB, −45°.</p>
    </td>
    <td width="50%" valign="top">
      <h3>📉 DC tarama</h3>
      <img src="docs/images/dc-sweep.png" alt="LED devresinde kaynak gerilimine karşı akım eğrisi" />
      <p>Bir kaynağın değerini tarayarak karakteristik eğri çıkar; verileri CSV olarak al.</p>
    </td>
  </tr>
  <tr>
    <td width="50%" valign="top">
      <h3>🎛️ Canlı kontrol</h3>
      <img src="docs/images/live-control.png" alt="Potansiyometre kaydırılarak lamba parlaklığı değiştiriliyor" />
      <p>Potansiyometreyi kaydır, butona bas, anahtarı aç-kapat — simülasyon yeniden başlamadan.</p>
    </td>
    <td width="50%" valign="top">
      <h3>🗂️ Örnek galerisi</h3>
      <img src="docs/images/gallery.png" alt="12 hazır örnek devrenin önizlemeli galerisi" />
      <p>12 hazır devre: LED'den 555'e, op-amp'ten onaltılık sayıcıya. Tek tıkla aç, F5'e bas.</p>
    </td>
  </tr>
</table>

### Tüm özellikler

<details open>
<summary><b>Şema editörü</b></summary>

- Kenar çubuğundan tıklayarak veya sürükleyip bırakarak bileşen yerleştirme
- Pinden tel çekme, köşeleri otomatik L biçiminde yönlendirme, T bağlantıları
- Seçim, kutuyla seçim, taşıma (bağlı teller esneyerek takip eder), döndürme
- Geri al / yinele, yakınlaştırma, kaydırma, devreyi ekrana sığdırma
- SI önekli değerler: `4k7`, `100n`, `2.2µ`, `4,7k` (Türkçe ondalık virgül de olur)
- Boşta kalan pinlerin ve geçersiz değerlerin anında işaretlenmesi

</details>

<details>
<summary><b>Simülasyon ve görselleştirme</b></summary>

- Gerçek zamanlı geçici durum simülasyonu, ayarlanabilir hız (1× … 1/100000×)
- DC çalışma noktası, AC analizi (Bode), DC tarama
- Gerilime göre renklenen teller, akım yönü ve şiddetini gösteren hareketli noktalar
- Parlayan LED'ler ve lambalar, 7 segment gösterge, lojik problar
- Telin veya pinin üzerine gelince o noktanın gerilimi
- Anlaşılır hata mesajları: toprak yok, kısa devre, geçersiz değer…

</details>

<details>
<summary><b>Dosya ve dışa aktarma</b></summary>

- `.dvrm` proje dosyaları (okunabilir JSON, sürüm kontrolüne uygun)
- SPICE netlist (`.cir`), PNG, SVG ve CSV dışa aktarımı
- Türkçe ve İngilizce arayüz
- Otomatik güncelleme (GitHub Releases üzerinden)

</details>

---

## 📥 İndirme

En son sürümü [**Releases**](../../releases/latest) sayfasından indir:

| Dosya                        | Açıklama                      |
| ---------------------------- | ----------------------------- |
| `Devrem_x.y.z_x64-setup.exe` | Önerilen kurulum dosyası      |
| `Devrem_x.y.z_x64_en-US.msi` | Kurumsal / toplu kurulum için |

**Gereksinimler:** Windows 10 (1809+) veya Windows 11, 64 bit. Microsoft Edge
WebView2 gerekir; Windows 11'de yüklü gelir, gerekirse kurulum dosyası indirir.

> [!NOTE]
> Kurulum dosyası henüz bir kod imzalama sertifikasıyla imzalanmadığı için
> Windows SmartScreen ilk kurulumda "Windows bilgisayarınızı korudu" uyarısı
> gösterebilir. **Ek bilgi → Yine de çalıştır** ile devam edebilirsin.

---

## 🚀 Hızlı Başlangıç

1. **Bileşen yerleştir** — soldaki listeden bir bileşen seç ve tuvale tıkla. `R` ile döndür, `Esc` ile bitir.
2. **Tel çek** — bir pine tıkla, köşeler için tıklamaya devam et; başka bir pine veya tele tıklayınca tel biter.
3. **Toprak ekle** — analog devrelerde bir **Toprak** sembolü gerekir.
4. **Çalıştır** — `F5`'e bas. Anahtarlara tıkla, potansiyometreyi kaydır, ölçü aletlerini izle.
5. **Analiz et** — alt paneldeki **Osiloskop**, **AC Analizi** ve **DC Tarama** sekmelerini kullan.

> [!TIP]
> Hemen denemek için **Dosya → Örnek Devreler…** menüsünden bir devre aç ve `F5`'e bas.

<details>
<summary><b>Klavye kısayolları</b></summary>

| Kısayol                        | İşlev                       |
| ------------------------------ | --------------------------- |
| `F5`                           | Simülasyonu başlat / durdur |
| `R`                            | Döndür                      |
| `W`                            | İmleçten tel çizmeye başla  |
| `Del`                          | Sil                         |
| `Esc`                          | İptal / seçimi kaldır       |
| `Ctrl+Z` / `Ctrl+Y`            | Geri al / yinele            |
| `Ctrl+A`                       | Tümünü seç                  |
| `Ctrl+N` / `Ctrl+O` / `Ctrl+S` | Yeni / aç / kaydet          |
| `Ctrl+0`                       | Devreyi ekrana sığdır       |
| Tekerlek · `Boşluk`+sürükle    | Yakınlaştır · kaydır        |

</details>

<details>
<summary><b>Değer yazımı</b></summary>

| Yazım                 | Değer                      |
| --------------------- | -------------------------- |
| `4k7`, `4.7k`, `4,7k` | 4,7 kΩ                     |
| `100n`                | 100 nF                     |
| `2.2µ`, `2.2u`        | 2,2 µ                      |
| `1M` / `1meg`         | 1 mega                     |
| `1m`                  | 1 mili                     |
| `10kΩ`, `5V`, `50Hz`  | birim eklemek isteğe bağlı |

</details>

---

## 🧩 Bileşenler

**39 bileşen**, dokuz kategoride:

| Kategori            | Bileşenler                                                        |
| ------------------- | ----------------------------------------------------------------- |
| **Pasifler**        | Direnç · kondansatör · bobin · potansiyometre                     |
| **Kaynaklar**       | DC gerilim · DC akım · AC (sinüs) · kare dalga · toprak           |
| **Yarı iletkenler** | Diyot · LED (5 renk) · zener · NPN · PNP · N-MOSFET · P-MOSFET    |
| **Entegre**         | Op-amp · 555 zamanlayıcı                                          |
| **Anahtarlar**      | Anahtar · buton · iki yollu (SPDT) anahtar                        |
| **Lojik kapılar**   | AND · OR · NOT · NAND · NOR · XOR · XNOR · hex → 7 segment çözücü |
| **Ardışıl lojik**   | D · JK · T flip-flop                                              |
| **Giriş / çıkış**   | Lojik giriş · lojik prob · saat üreteci · lamba · 7 segment       |
| **Ölçüm**           | Voltmetre · ampermetre                                            |

## 📊 Analizler

| Analiz                 | Ne yapar                                                                      |
| ---------------------- | ----------------------------------------------------------------------------- |
| **Gerçek zamanlı**     | Devre sürekli çalışır; değişiklikler anında yansır, etkileşimler durumu korur |
| **DC çalışma noktası** | Tüm düğüm gerilimleri ve akımlar (durağan durum)                              |
| **AC analizi**         | Frekans taraması; kazanç (dB) ve faz (°) — Bode diyagramı                     |
| **DC tarama**          | Bir kaynağı tarayıp ölçü aletlerinin okumalarını çizer                        |
| **Osiloskop**          | Ölçü aletlerinin zamana göre değişimi; pencere süresi ayarlanabilir           |

## 🗂️ Örnek Devreler

[`examples/`](examples/) klasöründe ve uygulamadaki galeride:

| Örnek                                                     | Gösterdiği                           |
| --------------------------------------------------------- | ------------------------------------ |
| [LED Devresi](examples/led.dvrm)                          | Akım sınırlayıcı direnç, ampermetre  |
| [Gerilim Bölücü](examples/gerilim-bolucu.dvrm)            | İki dirençle gerilim bölme           |
| [RC Şarj](examples/rc-sarj.dvrm)                          | Kondansatörün gerçek zamanlı dolması |
| [AC ile Yanıp Sönen LED](examples/ac-led.dvrm)            | Sinüs kaynağın yarım dalgası         |
| [Işık Ayarı](examples/isik-ayari.dvrm)                    | Potansiyometreyle canlı parlaklık    |
| [Transistörlü Anahtar](examples/transistor-anahtar.dvrm)  | NPN transistörle LED sürme           |
| [Zener Regülatörü](examples/zener-regulator.dvrm)         | 5,1 V'ta sabit çıkış                 |
| [Eviren Yükselteç](examples/evirici-yukselteci.dvrm)      | Op-amp, −10 kazanç, osiloskop        |
| [RC Alçak Geçiren Filtre](examples/rc-alcak-geciren.dvrm) | AC analizi, Bode diyagramı           |
| [555 Flaşör](examples/555-flasor.dvrm)                    | Kararsız (astable) 555               |
| [2 Bitlik Sayıcı](examples/sayici.dvrm)                   | T flip-flop'larla ikili sayma        |
| [Onaltılık Sayıcı](examples/hex-sayac.dvrm)               | Çözücü ve 7 segment gösterge         |

<details>
<summary><b>Dışa aktarılmış şema örneği (PNG)</b></summary>
<br />
<img src="docs/images/export.png" alt="555 flaşör devresinin PNG olarak dışa aktarılmış şeması" width="560" />
</details>

---

## 🛠 Kaynaktan Derleme

### Gereksinimler

- [Rust](https://rustup.rs/) (stable)
- [Node.js](https://nodejs.org/) 20+ ve [pnpm](https://pnpm.io/)
- Windows için: Microsoft C++ Build Tools ve WebView2 Runtime
  ([Tauri ön koşulları](https://v2.tauri.app/start/prerequisites/))

### Komutlar

```bash
# Bağımlılıkları yükle
pnpm install

# Masaüstü uygulamayı geliştirme modunda çalıştır
pnpm dev

# Testler ve kontroller
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
pnpm typecheck && pnpm lint && pnpm format:check

# Kurulum paketi (NSIS + MSI) — imza anahtarı için docs/release.md
pnpm build

# Web sitesi (http://localhost:3000)
pnpm --filter web dev
```

### Depo yapısı

```
crates/
  devrem-core/          Belge, netlist, SI birimleri, bileşen türleri
  devrem-components/    Cihaz modelleri (diyot, BJT, MOSFET, op-amp, lojik, 555)
  devrem-sim/           MNA simülatörü, analizler, SPICE dışa aktarımı
apps/
  desktop/              Tauri 2 + React masaüstü uygulaması
  web/                  Next.js tanıtım sitesi
packages/               Ortak bileşen tanımları ve tasarım token'ları
locales/                Türkçe ve İngilizce çeviriler
examples/               Örnek devreler (.dvrm)
docs/                   Mimari, sürüm yayınlama, görseller
```

---

## 🏗 Mimari

```mermaid
flowchart LR
    UI["Arayüz<br/><sub>React · TypeScript · Canvas 2D</sub>"]
    IPC["Tauri<br/><sub>komutlar + gerçek zamanlı oturum</sub>"]
    SIM["devrem-sim<br/><sub>MNA · Newton-Raphson · LU</sub>"]
    CORE["devrem-core<br/><sub>netlist · .dvrm</sub>"]
    COMP["devrem-components<br/><sub>cihaz modelleri</sub>"]

    UI -- "komut" --> IPC
    IPC -- "~30 kare/s" --> UI
    IPC --> SIM
    CORE --> SIM
    COMP --> SIM
```

Şema, netlist üreticisinde elektriksel düğümlere dönüştürülür; her bileşen
MNA matrisine kendi katkısını ekler. Diyot, transistör ve op-amp gibi doğrusal
olmayan bileşenler Newton-Raphson ile çözülür; kondansatör ve bobinler trapez
yöntemiyle ayrıklaştırılır. Lojik bileşenler aynı matriste, gerilim eşiğiyle
okunan girişler ve 0 / 5 V çıkışlarla çalışır.

Ayrıntılar: [**docs/ARCHITECTURE.md**](docs/ARCHITECTURE.md)

| Katman     | Teknoloji                                                                 |
| ---------- | ------------------------------------------------------------------------- |
| Motor      | Rust · serde · num-complex · thiserror · tracing                          |
| Masaüstü   | Tauri 2 (updater, dialog)                                                 |
| Arayüz     | React 19 · TypeScript · Vite · Zustand · Tailwind CSS 4 · uPlot · i18next |
| Web sitesi | Next.js 16 (statik dışa aktarım) · Tailwind CSS 4                         |
| CI / sürüm | GitHub Actions · tauri-action                                             |

---

## 🗺 Yol Haritası

- [x] Şema editörü, gerçek zamanlı simülasyon, akım animasyonu
- [x] DC çalışma noktası, AC analizi, DC tarama, osiloskop
- [x] Transistörler, op-amp, 555, lojik kapılar, flip-flop'lar, 7 segment
- [x] SPICE / PNG / SVG / CSV dışa aktarımı, örnek galerisi
- [x] Kurulum paketi ve otomatik güncelleme altyapısı
- [ ] İlk sürümün yayımlanması
- [ ] Yeni bileşenler: transformatör, kontrollü kaynaklar, röle, kaydırmalı yazmaç
- [ ] Büyük devreler için seyrek (sparse) çözücü
- [ ] Kopyala / yapıştır, alt devreler
- [ ] Windows kod imzalama

Fikrin mi var? Bir [issue](../../issues) aç.

---

## 🤝 Katkıda Bulunma

Hata bildirimleri, öneriler ve kod katkıları memnuniyetle karşılanır.

1. Bir hata veya öneri için önce bir [issue](../../issues) aç.
2. Depoyu çatalla (_fork_), bir özellik dalı oluştur ve
   [Conventional Commits](https://www.conventionalcommits.org/) biçiminde commit at
   (`feat:`, `fix:`, `docs:` …).
3. Göndermeden önce kontrolleri çalıştır:

   ```bash
   cargo fmt --all -- --check
   cargo clippy --workspace --all-targets -- -D warnings
   cargo test --workspace
   pnpm typecheck && pnpm lint && pnpm format:check
   ```

4. Bir _pull request_ aç.

Yeni bir bileşen eklemek için başlangıç noktası
[docs/ARCHITECTURE.md](docs/ARCHITECTURE.md). Sürüm yayınlama adımları
[docs/release.md](docs/release.md)'de, değişiklikler [CHANGELOG.md](CHANGELOG.md)'de.

Katkıda bulunarak, katkının proje ile aynı MIT lisansı altında yayımlanmasını
kabul etmiş olursun.

---

## 🔒 Gizlilik

Devrem **hesap gerektirmez ve veri toplamaz.** Devreler yalnızca senin
bilgisayarında `.dvrm` dosyası olarak saklanır. Uygulamanın yaptığı tek ağ
isteği, açılışta GitHub Releases'tan yeni sürüm olup olmadığını kontrol
etmektir.

## 📄 Lisans

[MIT](LICENSE) © 2026 HamzaShn

Devrem'i ücretsiz olarak kullanabilir, kopyalayabilir, değiştirebilir ve
dağıtabilirsin; lisans metnini ve telif bildirimini korumak yeterlidir.

<div align="center">
<br />
<sub><b>Devrem</b> — Devreni çiz, çalıştır, anla. ⚡</sub>
</div>
