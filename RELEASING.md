# Procedimento de Lançamento (Release Guide) — Nixo

Este documento descreve o fluxo passo a passo para gerar, testar e publicar uma nova versão oficial do **Nixo Media Player**.

---

## 1. Visão Geral do Ciclo de Lançamento

O processo de publicação do Nixo segue quatro etapas sequenciais:

```text
[1. Validação Técnica] ➔ [2. Bump de Versão] ➔ [3. Smoke Test Local] ➔ [4. Tag Git & CI/CD]
```

O CI/CD no **GitHub Actions** (`.github/workflows/release.yml`) é acionado automaticamente a cada nova tag que inicia com `v*` (por exemplo, `v0.1.0`), gerando os artefatos para Windows e Linux com seus respectivos checksums SHA-256.

---

## 2. Checklist Pré-Lançamento

Execute todas as validações obrigatórias no terminal para garantir conformidade estrita com o CI:

### 2.1 Testes Unitários e de Integração
```powershell
cargo test --workspace
```
*Critério:* Todos os testes devem passar com `0 failed`.

### 2.2 Verificação Estrita do Clippy
```powershell
cargo clippy --workspace --all-targets -- -D warnings
```
*Critério:* Zero advertências (warnings).

### 2.3 Formatação de Código
```powershell
cargo fmt --all -- --check
```
*Critério:* Formatação idêntica ao padrão Rustfmt.

---

## 3. Bump de Versão e Documentação

Ao definir a nova versão semântica (`vMAJOR.MINOR.PATCH`):

1. **`Cargo.toml` (raiz do workspace)**:
   Atualize o campo `version` em `[workspace.package]`:
   ```toml
   [workspace.package]
   version = "0.1.0"
   ```

2. **`docs/index.html` (Landing Page)**:
   Atualize o badge de versão no rodapé (`<footer>`):
   ```html
   <span class="font-mono-timestamp text-xs ...">v0.1</span>
   ```

3. **`CHANGELOG.md`**:
   Documente as novidades agrupadas pelas categorias recomendadas:
   - `### Added`: Novos recursos implementados.
   - `### Changed`: Melhorias de interface ou comportamentos ajustados.
   - `### Fixed`: Bugs corrigidos.

4. **Commit de Atualização**:
   ```bash
   git add Cargo.toml Cargo.lock docs/index.html CHANGELOG.md
   git commit -m "chore(release): bump version to vX.Y.Z"
   git push origin main
   ```

---

## 4. Teste de Empacotamento Local (Smoke Test no Windows)

Antes de criar a tag pública, gere os artefatos localmente para validação manual:

### 4.1 Gerar o Pacote Portátil (.zip)
```powershell
powershell -ExecutionPolicy Bypass -File ./scripts/package.ps1
```
*Gera:* `dist/nixo-vX.Y.Z-windows-x86_64.zip` e `.sha256`.

### 4.2 Gerar o Instalador Executável (.exe)
```powershell
powershell -ExecutionPolicy Bypass -File ./scripts/build-installer.ps1
```
*Gera:* `dist/nixo-vX.Y.Z-windows-x86_64-setup.exe` e `.sha256`.

### 4.3 Checklist Manual do Instalador
- [ ] O banner lateral verde do Nixo aparece corretamente no assistente.
- [ ] A instalação conclui sem exigir privilégios de Administrador (`AppData\Local\Programs\Nixo`).
- [ ] O atalho do Menu Iniciar e da Área de Trabalho abre o player.
- [ ] Clicar com o botão direito em um arquivo `.mp4` exibe o Nixo no menu **"Abrir com..."**.
- [ ] O desinstalador remove os arquivos e entradas do registro de forma limpa.

---

## 5. Publicação Oficial (Git Tag & CI/CD)

Com a branch `main` sincronizada e testada:

### 5.1 Criar a Tag Anotada
```bash
git tag -a v0.1.0 -m "Nixo v0.1.0 — Initial Release"
```

### 5.2 Enviar a Tag ao GitHub
```bash
git push origin v0.1.0
```

---

## 6. Pós-Lançamento no GitHub Actions

Assim que a tag for enviada, o workflow **Release** iniciará automaticamente:

1. **Acompanhar o Build**:
   Acesse `https://github.com/carllosnc/nixo/actions` e acompanhe os jobs:
   - `release-windows`: Compila em Release com LTO, gera o `.zip` portátil e o instalador Inno Setup `.exe`.
   - `release-linux`: Compila para Linux x86_64 e gera o `.tar.gz`.

2. **Revisar a Release no GitHub**:
   Acesse `https://github.com/carllosnc/nixo/releases/tag/v0.1.0`:
   - Verifique se os 6 artefatos estão anexados:
     - `nixo-vX.Y.Z-windows-x86_64-setup.exe`
     - `nixo-vX.Y.Z-windows-x86_64-setup.exe.sha256`
     - `nixo-vX.Y.Z-windows-x86_64.zip`
     - `nixo-vX.Y.Z-windows-x86_64.zip.sha256`
     - `nixo-vX.Y.Z-linux-x86_64.tar.gz`
     - `nixo-vX.Y.Z-linux-x86_64.tar.gz.sha256`
   - Clique em **Edit release** para adicionar a descrição formatada e publicar oficialmente.
