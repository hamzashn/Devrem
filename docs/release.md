# Sürüm Yayınlama

Devrem, Windows kurulum paketlerini GitHub Releases üzerinden dağıtır. Uygulama
açılışta en son sürümün `latest.json` dosyasını kontrol eder ve yeni sürüm varsa
durum çubuğunda "Güncelle ve yeniden başlat" düğmesi gösterir.

## Bir kerelik kurulum

1. **GitHub deposunu oluştur** ve kodu gönder.
2. **Güncelleme adresini ayarla:**
   `apps/desktop/src-tauri/tauri.conf.json` → `plugins.updater.endpoints` içindeki
   `OWNER/devrem` kısmını kendi `kullanıcı/depo` adınla değiştir.
3. **Güncelleme imza anahtarını gizli değişken olarak ekle**
   (Settings → Secrets and variables → Actions):

   | Ad                                   | Değer                                    |
   | ------------------------------------ | ---------------------------------------- |
   | `TAURI_SIGNING_PRIVATE_KEY`          | `~/.tauri/devrem.key` dosyasının içeriği |
   | `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | Boş bırak (anahtar parolasız üretildi)   |

   > ⚠️ `devrem.key` gizli anahtardır; depoya eklenmemeli ve kaybedilmemelidir.
   > Kaybedilirse yayımlanmış uygulamalar yeni güncellemeleri doğrulayamaz.
   > Bir yedeğini güvenli bir yerde sakla. Açık anahtar (`devrem.key.pub`)
   > `tauri.conf.json` içinde zaten kayıtlıdır.

## Yeni sürüm çıkarmak

1. Sürüm numarasını üç yerde güncelle:
   - `Cargo.toml` → `[workspace.package] version`
   - `apps/desktop/src-tauri/tauri.conf.json` → `version`
   - `apps/desktop/package.json` → `version`
2. `CHANGELOG.md` dosyasına değişiklikleri yaz.
3. Etiketi gönder:

   ```bash
   git tag v0.2.0
   git push origin v0.2.0
   ```

4. **Sürüm** iş akışı NSIS (`.exe`) ve MSI paketlerini, imzalarını ve
   `latest.json` dosyasını taslak sürüm olarak yükler.
5. GitHub'da taslağı kontrol edip **Publish release** ile yayımla. Yayımlanınca
   mevcut kullanıcılar güncellemeyi görmeye başlar.

## Yerelde kurulum paketi üretmek

```bash
# Git Bash
export TAURI_SIGNING_PRIVATE_KEY="$(cat ~/.tauri/devrem.key)"
export TAURI_SIGNING_PRIVATE_KEY_PASSWORD=""
pnpm build
```

Çıktılar: `target/release/bundle/nsis/` ve `target/release/bundle/msi/`.

## Windows kod imzalama (sertifika alındığında)

Şu an kurulum paketleri **imzasızdır**; Windows SmartScreen ilk kurulumda
"Tanınmayan yayımcı" uyarısı gösterir. Bir kod imzalama sertifikası (OV veya EV)
alındığında:

1. Sertifikayı `.pfx` olarak dışa aktar, base64'e çevir ve
   `WINDOWS_CERTIFICATE` / `WINDOWS_CERTIFICATE_PASSWORD` gizli değişkenleri
   olarak ekle.
2. `release.yml` içinde, `tauri-action` adımından önce sertifikayı
   `Import-PfxCertificate` ile içe aktaran bir adım ekle.
3. `tauri.conf.json` → `bundle.windows` altına sertifikanın
   `certificateThumbprint`, `digestAlgorithm: "sha256"` ve
   `timestampUrl` değerlerini yaz.

Bulut tabanlı imzalama (ör. Azure Trusted Signing) kullanılacaksa bunun yerine
`bundle.windows.signCommand` ayarlanır. Ayrıntılar:
<https://v2.tauri.app/distribute/sign/windows/>
