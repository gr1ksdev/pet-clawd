# Pet Clawd — App Android (Rust/Tauri 2)

Port do plugin **clawd_pet_pt.elyx** (exteraGram) para um app Android independente.
Núcleo 100% **Rust** (estado do pet, IA, persistência) + WebView com o visual
liquid glass original do plugin (HTML/CSS/JS reutilizado).

## Estrutura
```
clawd_pet_app/
  app-icon.png              ícone fonte (pixel-art do Clawd, 1024×1024)
  ui/index.html             interface completa (temas, mochila, vínculo, IA)
  src-tauri/
    src/brain.rs            port do ClawdBrain: humor, vínculo, fome, energia,
                            sujeiras, mochila com reposição diária, decay por
                            timestamp (sem serviço em background)
    src/ai.rs               Gemini + OpenAI-compatível (OpenRouter/Groq/Ollama)
                            com fallback ai2
    src/store.rs            persistência JSON (state.json / settings.json)
    src/commands.rs         comandos Tauri expostos à UI
    tauri.conf.json
  .github/workflows/android.yml  build do APK no GitHub Actions
```

## Como gerar o APK (GitHub Actions — recomendado)
Este projeto foi escrito em um aparelho Android (Arch ARM em proot), onde o
NDK oficial (binários x86_64) não roda. O build do APK é feito no CI:

1. Crie um repositório no GitHub e suba esta pasta;
2. A action `Android APK` roda a cada push (ou manualmente em *Run workflow*);
3. Baixe o artefato `PetClawd-debug-apk` e instale o `.apk` no aparelho.

## Como gerar o APK localmente (máquina Linux x86_64)
Requisitos: Rust stable (+ targets Android), JDK 17+, Android SDK/NDK,
`cargo install tauri-cli --version "^2"`.

```bash
./build.sh            # gera gen/ android e compila APK debug (aarch64)
```
O APK sai em `src-tauri/gen/android/app/build/outputs/apk/debug/`.

## Diferenças em relação ao plugin
| Plugin (exteraGram) | App |
|---|---|
| Janela central + mini flutuante sobre o Telegram | Tela própria do app (overlay fica para a v2) |
| Backup `.clawd` nas Mensagens Salvas | Backup `.clawd` local (app data dir) |
| Settings do host (chave/modelo de IA) | Aba Ajustes do app |
| Fala do Clawd via IA (opcional) | Mantido — Gemini/OpenAI/compatível + fallback |
| Devorar ícones/gravidade | Ficam para a v2 (dependem de overlay) |

## Roadmap v2
- Mini-janela flutuante real (permissão `SYSTEM_ALERT_WINDOW`);
- Backup com share sheet direto;
- Assinatura release + ícone adaptativo oficial.
