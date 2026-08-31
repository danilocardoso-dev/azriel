# AZRIEL v0.5 — Knowledge Core

A versão **v0.4 — HUD Vivo** foi concluída.

Agora implemente:

# v0.5 — Knowledge Core

Antes de modificar qualquer arquivo:

1. leia completamente o `README.md`;
2. leia `docs/roadmap.md`, se existir;
3. leia a documentação da v0.4;
4. examine a arquitetura atual;
5. identifique componentes, tipos e mocks existentes;
6. execute o projeto e os testes disponíveis;
7. preserve tudo que estiver funcionando.

Não refatore partes estáveis apenas por preferência estética.

---

# Objetivo

A v0.5 deve transformar os dados atualmente mockados do Azriel em **dados persistentes e estruturados**.

Ao final desta versão:

> projetos, conhecimentos, formação e métricas devem sobreviver ao fechamento e reinício do aplicativo.

O Azriel deve possuir seu primeiro banco de conhecimento real.

---

# Tecnologia

Utilizar:

**SQLite**

O banco deve ser local.

Integrar SQLite através da arquitetura Tauri existente.

Não criar backend HTTP separado.

Fluxo esperado:

```text
React / TypeScript
        │
        ▼
Service / Repository
        │
        ▼
Tauri
        │
        ▼
SQLite
```

Evitar acesso ao banco diretamente dentro dos componentes React.

---

# Princípio arquitetural

A UI NÃO deve conhecer detalhes de SQLite.

Criar uma camada de acesso a dados.

Exemplo conceitual:

```text
UI
 │
 ▼
Services
 │
 ▼
Repositories
 │
 ▼
Database
```

Assim, componentes como:

* StarkMap;
* Projects;
* Knowledge;
* Education;
* GapDiagnostics;

consomem dados através de uma API interna clara.

---

# Banco de dados

Criar schema inicial para pelo menos:

## knowledge_areas

Representa áreas de conhecimento.

Campos conceituais:

```text
id
name
category
description
coverage
depth
priority
created_at
updated_at
```

---

## projects

Representa projetos.

Campos:

```text
id
name
description
objective
category
status
created_at
updated_at
```

Status possíveis:

```text
active
research
paused
planned
completed
```

---

## project_knowledge

Relacionamento N:N entre projetos e conhecimentos.

```text
project_id
knowledge_id
```

Exemplo:

```text
GeneScope
   │
   ├── Genética
   ├── Biologia Molecular
   ├── Bioinformática
   └── Programação
```

---

## education

Representa formação acadêmica.

Campos conceituais:

```text
id
name
type
institution
status
start_date
expected_end_date
completed_at
description
```

Tipos possíveis:

```text
graduation
postgraduate
masters
doctorate
course
certification
```

Não exigir instituição caso ela ainda não esteja registrada.

---

## knowledge_history

Essa tabela é importante.

Ela permitirá acompanhar evolução histórica.

```text
id
knowledge_id
coverage
depth
recorded_at
reason
```

Exemplo:

```text
Eletrônica

2026-08
Cobertura: 20
Profundidade: 10

2027-02
Cobertura: 28
Profundidade: 15
```

Isso será usado futuramente para gráficos de evolução.

---

# Integração

A métrica global de integração deve continuar existindo.

Nesta versão, NÃO criar um algoritmo sofisticado ou pseudo-inteligente para calculá-la.

Se necessário, manter integração como configuração/métrica persistida separadamente.

Documentar que sua fórmula será refinada posteriormente.

---

# Seed inicial

Na primeira execução, popular o banco com os dados atualmente existentes nos mocks.

IMPORTANTE:

O seed deve ocorrer somente quando necessário.

Não duplicar registros a cada inicialização.

---

# Projetos iniciais

Persistir:

## Azriel

Sistema pessoal de inteligência, conhecimento, pesquisa e automação.

Status:

`active`

---

## ArcCore

Projeto experimental de energia, armazenamento, gerenciamento e controle.

Status:

`research`

---

## Mendel Lab

Áreas relacionadas:

* Genética
* Biologia
* Biomedicina
* Biotecnologia
* Probabilidade
* Estatística
* Programação

---

## Gene Expression Explorer

Áreas:

* Genética
* Biologia Molecular
* Biotecnologia
* Bioinformática
* Transcriptômica
* Análise de Dados
* Programação

---

## PCR Simulator

Áreas:

* Biologia Molecular
* Genética
* Biotecnologia
* Bioinformática
* Programação

---

## GeneScope

Áreas:

* Genética
* Biologia Molecular
* Biotecnologia
* Bioinformática
* Programação

---

## Atlas3D

Status:

`paused`

---

# Formação

Persistir as três pós-graduações em andamento:

## Biotecnologia

Status:

`in_progress`

Conclusão prevista:

**novembro de 2026**

---

## Internet das Coisas — IoT

Status:

`in_progress`

Conclusão prevista:

**novembro de 2026**

---

## Big Data Analytics

Status:

`in_progress`

Conclusão prevista:

**novembro de 2026**

---

Persistir também:

## Biomedicina

Tipo:

`graduation`

Status:

`planned`

Previsão de início:

**2027**

---

## Engenharia Mecatrônica

Tipo:

`graduation`

Status:

`planned`

---

## Mestrado interdisciplinar

Tipo:

`masters`

Status:

`planned`

Área ainda não definida.

---

# Conhecimentos

Migrar para SQLite as áreas atualmente utilizadas pelo Mapa Stark e Knowledge Core.

Pelo menos:

* Cibersegurança
* Programação
* Inteligência Artificial
* Biologia
* Genética
* Biologia Molecular
* Biomedicina
* Biotecnologia
* Bioinformática
* Matemática
* Estatística
* Big Data
* IoT
* Física Aplicada
* Eletrônica
* Engenharia Elétrica
* Engenharia Mecânica
* Energia
* Controle
* Automação
* Robótica
* Materiais

Preservar os valores atuais de cobertura e profundidade existentes na v0.4.

Não inventar novos valores apenas para preencher campos.

---

# CRUD

Implementar operações básicas para:

## Conhecimentos

* listar;
* buscar por ID;
* criar;
* atualizar;
* remover.

## Projetos

* listar;
* buscar;
* criar;
* atualizar;
* alterar status;
* remover.

## Formação

* listar;
* criar;
* atualizar;
* remover.

Não é necessário construir formulários complexos para tudo nesta versão.

Priorizar arquitetura e persistência.

---

# Atualização de conhecimento

Criar uma operação específica para atualizar:

* cobertura;
* profundidade.

Exemplo conceitual:

```ts
updateKnowledgeMetrics({
  knowledgeId,
  coverage: 30,
  depth: 18,
  reason: "Estudo de circuitos básicos"
})
```

Quando uma métrica for alterada:

1. atualizar `knowledge_areas`;
2. registrar automaticamente um snapshot em `knowledge_history`.

Não depender da UI para criar o histórico.

Essa regra deve estar na camada de domínio/service.

---

# Mapa Stark

O `StarkMap` deve parar de consumir mocks.

Ele deve carregar informações persistidas.

Fluxo:

```text
SQLite
   ↓
Repository
   ↓
Service
   ↓
React
   ↓
StarkMap
```

Alterações realizadas nos conhecimentos devem refletir no gráfico.

---

# Projetos

A tela de projetos deve consumir SQLite.

Ao abrir um projeto, carregar também seus conhecimentos relacionados.

Exemplo:

```text
PCR Simulator
│
├── Biologia Molecular
├── Genética
├── Biotecnologia
├── Bioinformática
└── Programação
```

---

# Knowledge Core

A página de conhecimento deve passar a representar dados reais do banco.

Mostrar:

* nome;
* categoria;
* cobertura;
* profundidade;
* prioridade;
* projetos relacionados.

---

# Gap Diagnostics

O diagnóstico deve ser derivado dos conhecimentos persistidos.

Não manter uma segunda lista manual de lacunas se ela puder ser derivada dos dados.

Criar uma regra simples e documentada.

Por exemplo:

lacuna pode considerar diferença entre uma meta e:

* cobertura;
* profundidade;
* prioridade.

Não criar algoritmo excessivamente complexo nesta versão.

O objetivo é eliminar duplicação de dados.

---

# Histórico

Criar uma visualização inicial do histórico de uma área.

Ao selecionar um conhecimento, permitir visualizar algo como:

```text
ELETRÔNICA

Cobertura atual: 20%
Profundidade atual: 10%

Histórico

Ago/2026    20 / 10
Set/2026    24 / 12
Out/2026    30 / 18
```

Se houver apenas um registro, mostrar apenas esse registro.

Não inventar histórico passado.

---

# Estado da aplicação

Adicionar estados apropriados:

* loading;
* error;
* empty;
* success.

A aplicação não deve assumir que o banco sempre responderá corretamente.

Falhas devem produzir mensagens compreensíveis.

---

# Dados mockados

Depois da migração:

não apagar imediatamente os arquivos de mocks caso ainda sejam úteis para desenvolvimento/testes.

Entretanto, a aplicação principal NÃO deve depender deles.

Se forem mantidos:

mover ou identificar claramente como fixtures/test data.

---

# Segurança

Nesta versão:

* banco exclusivamente local;
* nenhuma sincronização externa;
* nenhum servidor;
* nenhuma credencial;
* nenhuma informação sensível necessária.

Validar entradas antes de persistir.

Usar queries parametrizadas.

---

# Migrations

Criar sistema de migrations desde agora.

Não criar banco apenas executando vários `CREATE TABLE IF NOT EXISTS` espalhados pelo código.

Precisamos conseguir evoluir:

```text
v0.5
↓
v0.6
↓
v0.7
↓
1.0
```

sem destruir dados existentes.

Criar migration inicial claramente versionada.

---

# Backup

Não implementar sistema completo de backup ainda.

Entretanto:

documentar claramente onde o arquivo SQLite fica armazenado.

Isso será necessário futuramente.

---

# Estrutura sugerida

Adaptar à arquitetura existente.

Conceitualmente:

```text
src/
├── components/
├── pages/
├── services/
│   ├── knowledgeService.ts
│   ├── projectService.ts
│   └── educationService.ts
├── repositories/
│   ├── knowledgeRepository.ts
│   ├── projectRepository.ts
│   └── educationRepository.ts
├── types/
└── ...
```

A implementação concreta pode variar dependendo de como Tauri/SQLite estiverem configurados.

Não criar camadas artificiais se a biblioteca escolhida já resolver alguma delas elegantemente.

---

# TypeScript

Manter tipagem forte.

Evitar:

```ts
any
```

Criar tipos claros para:

* KnowledgeArea;
* KnowledgeHistory;
* Project;
* Education;
* ProjectStatus;
* EducationStatus;
* KnowledgePriority.

---

# Testes

Adicionar testes principalmente para regras que não dependam diretamente da UI.

Prioridades:

1. atualização de cobertura/profundidade;
2. criação automática do histórico;
3. relacionamento projeto ↔ conhecimento;
4. cálculo simples de lacunas;
5. seed sem duplicação.

---

# Interface

Preservar a identidade visual da v0.4.

Não redesenhar o HUD inteiro.

A v0.5 é principalmente uma versão de:

**dados + persistência + conhecimento.**

Adicionar somente UI necessária para demonstrar que a persistência funciona.

---

# Teste de persistência obrigatório

Validar manualmente este fluxo:

1. abrir Azriel;
2. alterar uma métrica;
3. fechar completamente o aplicativo;
4. abrir novamente;
5. confirmar que o valor continua alterado;
6. verificar que o histórico foi registrado.

Esse é um critério obrigatório da v0.5.

---

# Documentação

Criar:

`docs/versions/v0.5.md`

Documentar:

* objetivo;
* arquitetura;
* banco;
* tabelas;
* migrations;
* seed;
* serviços;
* persistência;
* limitações;
* decisões tomadas.

Atualizar:

`docs/roadmap.md`

marcando:

**v0.4 — concluída**

e:

**v0.5 — em desenvolvimento**

Não transformar o README em changelog.

---

# Critérios de aceite

A v0.5 só está concluída quando:

1. SQLite estiver funcionando localmente;
2. migrations existirem;
3. seed inicial funcionar sem duplicação;
4. projetos forem carregados do banco;
5. conhecimentos forem carregados do banco;
6. formação for carregada do banco;
7. relacionamentos projeto/conhecimento funcionarem;
8. StarkMap não depender mais de mocks;
9. Knowledge Core não depender mais de mocks;
10. atualização de cobertura funcionar;
11. atualização de profundidade funcionar;
12. mudanças criarem histórico automaticamente;
13. histórico sobreviver ao reinício;
14. projetos sobreviverem ao reinício;
15. formação sobreviver ao reinício;
16. Gap Diagnostics utilizar dados persistidos;
17. erros de banco forem tratados;
18. TypeScript não possuir erros;
19. testes relevantes passarem;
20. build funcionar;
21. Tauri iniciar corretamente;
22. a identidade visual da v0.4 permanecer intacta;
23. `docs/versions/v0.5.md` existir;
24. nenhuma funcionalidade da v0.4 sofrer regressão importante.

---

# Fora do escopo

NÃO implementar nesta versão:

* Ollama;
* chat com IA;
* LLM;
* reconhecimento de voz;
* automação do Windows;
* execução de comandos;
* MQTT;
* ESP32;
* dispositivos IoT;
* sincronização em nuvem;
* autenticação;
* API externa.

Esses recursos pertencem a versões posteriores.

---

# Resultado esperado

Ao terminar a v0.5, Azriel ainda não precisa “pensar”.

Mas ele precisa finalmente **lembrar**.

O HUD deixa de representar dados escritos manualmente no código e passa a ser uma visualização do estado real e persistente do sistema.

Esse é o primeiro passo para que, na v0.6, a inteligência artificial possa consultar e compreender o próprio Knowledge Core.
