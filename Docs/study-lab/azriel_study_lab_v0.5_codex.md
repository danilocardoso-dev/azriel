# AZRIEL STUDY LAB v0.5 — Study Library
## Materials, PDFs & Source-Aware Study — Prompt para Codex

## Contexto
v0.1 Foundation, v0.2 Knowledge Workspace, v0.3 Review e v0.4 AI Study Tools estão concluídas.

A v0.5 adiciona materiais externos sem transformar o Study Lab em RAG.

Objetivo: Biblioteca local confiável para PDFs, imagens, TXT/Markdown e links, relacionada a Roadmaps/Notes e utilizável explicitamente pelo AI Context Builder.

## 1. Princípio
Separar:
- MATERIAL = fonte externa
- NOTE = registro do usuário
- AI RESULT = proposta da IA
- STUDY CARD = revisão

Nunca converter silenciosamente um no outro.

## 2. Inspeção obrigatória
Antes de codificar, revisar v0.1–v0.4, filesystem/storage existente, Tauri file APIs, SQLite/migrations, AI ContextBuilder, Note/Card relations, suporte PDF/text/image, paths/security e design system. Repositório é fonte de verdade.

## 3. Preservação / Navegação
Não quebrar Roadmaps, StudySession/Pomodoro, Notes, Review, Scheduler, AI Tools ou Histórico.

Navegação:
HOJE | ROADMAPS | CADERNOS | BIBLIOTECA | REVISÃO | HISTÓRICO

## 4. StudyMaterial
Criar entidade conceitual:
- id
- title
- material_type
- storage_kind
- local_path nullable
- external_url nullable
- original_filename nullable
- mime_type/file_size/checksum nullable
- source_author/source_title/source_year nullable
- description nullable
- status
- timestamps

Tipos iniciais: PDF, IMAGE, TEXT, MARKDOWN, LINK, OTHER quando necessário.

Storage: MANAGED_COPY, LINKED_LOCAL_FILE, EXTERNAL_URL. Adaptar aos padrões reais.

## 5. Storage
Para managed copy, usar app-data apropriado do Tauri/OS, nunca source tree. Nome interno seguro/único; filename original fica em metadata.

Quando simples, calcular checksum para integridade/deduplicação. Versões diferentes continuam permitidas.

## 6. Relações
Materiais podem se relacionar opcionalmente com Roadmap/Stage/Topic/Activity/Notebook/Note. Escolher tabela relacional quando many-to-many for claramente melhor que várias nullable FKs.

Não criar tags/pastas complexas.

## 7. Import Flow
`ADICIONAR → VALIDAR → METADATA → COPIAR/VINCULAR → PERSISTIR → ASSOCIAR`.

Erros devem ser explícitos. Não persistir registro completo se cópia obrigatória falhar.

## 8. PDF
Importar/abrir/preview quando stack suportar. Persistir filename, size, pages e metadata disponíveis.

Extrair texto somente de PDF com text layer. **Sem OCR.**

Criar estado de extração:
- NOT_ATTEMPTED
- AVAILABLE
- TEXT_UNAVAILABLE
- FAILED

Artefato conceitual `MaterialTextContent`: material_id, status, text, extracted_at, extractor_version, char_count.

Versionar como `STUDY_MATERIAL_PDF_TEXT_V1`.

Não confundir PDF escaneado sem texto com erro.

## 9. Sem RAG
Texto extraído serve para visualização, busca, seleção manual e contexto explícito da IA.

Não criar embeddings, vector DB ou retrieval automático.

Se estrutura por páginas for necessária para navegação, isso não deve virar pipeline RAG.

## 10. Imagens / Texto / Links
IMAGE: preview + metadata; sem OCR e sem envio automático à IA.

TXT/MD: leitura/indexação textual; não converter automaticamente em Note. Pode oferecer `CRIAR NOTA A PARTIR DESTE MATERIAL` com confirmação.

LINK: title/url/description/relações; sem scraping/download automático; abrir externamente com segurança.

## 11. Biblioteca UI
Tela simples:
`BIBLIOTECA | + ADICIONAR MATERIAL | BUSCAR | TODOS/PDF/IMAGEM/TEXTO/LINK`

Item: título, tipo, contexto, data, extraction status quando relevante.

Material Detail: metadata, relações, preview, texto extraído e ações realmente suportadas:
ABRIR / ASSOCIAR / CRIAR NOTA / USAR COM AZRIEL / ARQUIVAR / REMOVER.

## 12. Busca
Buscar title, description, filename, author e extracted text disponível. FTS5 somente se integrar de forma limpa. Sem semantic search. Limitar resultados.

## 13. Source-Aware AI
Integrar `StudyMaterialContext` ao `StudyAIContextBuilder`.

Material só entra quando usuário explicitamente:
- clica `USAR COM AZRIEL`, ou
- inicia ação AI a partir do material.

Nunca enviar Biblioteca inteira.

Prompt deve separar SOURCE MATERIAL / USER NOTE / USER QUESTION e instruir modelo a preservar terminologia, não inventar conteúdo como se estivesse na fonte e sinalizar lacunas/inferências.

`StudyAIResult` registra material_ids usados e UI mostra `FONTES DO CONTEXTO`.

## 14. Seleção explícita
Quando viewer permitir, seleção de texto pode alimentar EXPLICAR/RESUMIR/ME TESTAR/GERAR CARDS.

Se material exceder context limit, não enviar tudo silenciosamente. Usuário seleciona trecho/páginas ou recebe aviso.

Page range pode ser implementado se extração por páginas existir sem complexidade excessiva. Isso é seleção explícita, não RAG.

## 15. Material → Cards / Notes
GENERATE_CARDS continua produzindo drafts revisáveis antes de StudyCard real e preserva source material relation.

`CRIAR NOTA` gera Note vinculada; não preencher automaticamente com resumo AI. Conteúdo só entra por ação explícita.

Activity View: `MATERIAIS RELACIONADOS`.
StudyNote: `MATERIAIS RELACIONADOS`.

Abrir material durante StudySession não cria outra sessão.

## 16. Status / remoção
Status: ACTIVE, ARCHIVED, MISSING para linked file ausente e ERROR quando necessário.

Distinguir ARCHIVE / REMOVE FROM LIBRARY / DELETE MANAGED FILE.

Delete físico exige confirmação. Relações históricas não podem quebrar.

## 17. Segurança
Validar paths e evitar traversal/filename injection. Usar APIs seguras Tauri/OS.

Definir file-size limits. Não carregar PDF enorme inteiro em RAM sem necessidade.

Não confiar apenas na extensão; validar MIME/tipo quando possível.

Checksum duplicado deve apontar para material existente em vez de copiar silenciosamente.

## 18. Mudança/reprocessamento
Linked local file alterado deve marcar extração potencialmente stale quando detectável. Oferecer `REPROCESSAR TEXTO`.

Não reprocessar Biblioteca inteira no startup.

Falhas devem limpar temporários/managed copies órfãs quando possível.

## 19. SQLite / Backend
Migrations incrementais. Entidades prováveis: study_materials, material_relations, material_text_content — adaptar ao schema real.

Índices somente conforme consultas.

Operações conceituais:
import_material, add_link_material, get/list/search/update/archive/remove, associate_material, extract/reprocess/get_material_text.

Adaptar naming/arquitetura.

## 20. Frontend / Performance
Reutilizar Tauri file picker/APIs. Estados: selecting/importing/extracting/ready/error.

Operações longas assíncronas; UI não bloqueia.

Não criar framework de upload web.

## 21. Observabilidade / privacidade
Logger existente: material id/type/import status/extraction status/duration/error code. Não logar texto integral extraído.

Materiais locais por padrão. AI usa apenas provider explicitamente configurado; nenhuma cloud nova e nenhum upload silencioso.

Managed files ficam em app-data previsível para backup futuro; não implementar backup agora.

## 22. Testes
Import:
- PDF válido/inválido
- duplicate/oversized
- image/text/markdown/link
- filename estranho
- rollback/cleanup/DB failure

PDF:
- text layer
- sem texto
- extraction failure
- reprocess
- extractor version
- large context

Relações:
- Roadmap/Topic/Activity/Notebook/Note
- IDs inválidos

AI Context:
- material explicitamente selecionado entra
- não selecionado não entra
- selected text priority
- page range quando houver
- source IDs
- truncation
- nenhum retrieval implícito

## 23. Fluxos integrados
`Roadmap → Activity → StudySession → Import PDF → Associate → Extract → Select excerpt → EXPLAIN → Create Note explicitly → Generate Card Drafts → Save selected → Review`.

`Library → Add Markdown → Search → Open → Associate Note → Use with Azriel`.

## 24. Fora do escopo
Não implementar:
- OCR
- RAG
- embeddings
- vector DB
- web scraping
- cloud storage
- autonomous ingestion
- multimodal AI automático
- Mapa Stark

## 25. Critérios de aceite
v0.5 concluída quando:
1. BIBLIOTECA existe;
2. v0.1–v0.4 permanecem funcionais;
3. StudyMaterial/storage semantics existem;
4. PDF/image/text/Markdown/link suportados conforme escopo;
5. managed copy usa app-data;
6. checksum/dedup funciona;
7. metadata/relações funcionam;
8. Detail/preview/abrir funcionam quando suportados;
9. PDF text extraction funciona em text-layer;
10. sem-texto é identificado sem OCR;
11. extraction status/version existem;
12. busca funciona;
13. Activity/Note mostram materiais;
14. ContextBuilder aceita somente material explícito;
15. source trace existe;
16. context limits/truncation funcionam;
17. AI cards continuam drafts;
18. path/file-size/MIME protections existem;
19. delete físico exige confirmação;
20. rollback/cleanup funciona;
21. UI não bloqueia;
22. nenhuma cloud/OCR/RAG/vector DB/embeddings entra;
23. migrations preservam dados;
24. testes/build passam;
25. Tauri inicia;
26. documentação existe.

## 26. Documentação
Criar `docs/study-lab/v0.5.md` (ou padrão real) documentando StudyMaterial, storage, paths, import, relations, extraction/version, search, Source-Aware AI, context limits, security, cleanup, testes e limitações.

Roadmap:
- v0.1 Study Foundation — COMPLETED
- v0.2 Knowledge Workspace — COMPLETED
- v0.3 Review & Active Recall — COMPLETED
- v0.4 AI Study Tools — COMPLETED
- v0.5 Study Library — CURRENT

**Não criar automaticamente v0.6.** Depois da v0.5, parar e usar o Study Lab antes de decidir nova expansão.

## 27. Entrega Codex
Resumir arquivos/migrations, storage escolhido, entidades, import/extraction, relações, UI, Source-Aware AI, security, testes, build/Tauri, limitações e itens deliberadamente não implementados.

## Resultado esperado
Antes:
`ROADMAP → STUDY → NOTE/CARDS → AI`

Depois:
`MATERIAL REAL → BIBLIOTECA → CONTEXTO EXPLÍCITO → STUDY/NOTE/AI/CARDS → REVIEW`

A v0.5 deve fechar a primeira base funcional do Study Lab. Materiais externos passam a fazer parte do estudo sem introduzir RAG prematuramente. Depois desta versão, usar o sistema e deixar necessidades reais determinarem qualquer v0.6.
