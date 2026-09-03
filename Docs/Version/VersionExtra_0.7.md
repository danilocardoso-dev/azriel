# AZRIEL — Engineering Core v0.7 — Assembly Intelligence

As versões anteriores do Engineering Core foram concluídas e validadas:

* v0.1 — Hand Interaction
* v0.2 — Spatial Manipulation
* v0.2.1 — Calibration Refinement
* v0.3 — Model Core
* v0.4 — Component Core
* v0.5 — Exploded View
* v0.6 — AI Integration

Agora implemente:

# Engineering Core v0.7 — Assembly Intelligence

---

# Objetivo

Adicionar ao Engineering Core uma camada semântica persistente capaz de representar:

* componentes;
* nomes semânticos;
* subsistemas;
* funções;
* relações;
* hierarquia;
* notas técnicas;
* classificação manual;
* contexto de engenharia para o AI Core.

Ao final desta versão, o Azriel deve conseguir diferenciar:

```text
Mesh_047
```

de:

```text
Rotor
```

quando o operador explicitamente classificar esse componente.

O sistema deve permitir ensinar a estrutura do assembly ao Azriel de forma persistente.

---

# Princípio central

A v0.7 deve provar:

# O AZRIEL CONSEGUE REPRESENTAR O SIGNIFICADO DOS COMPONENTES, NÃO APENAS SUA GEOMETRIA.

Não inventar semântica.

Não inferir função mecânica sem evidência.

---

# Antes de modificar código

1. leia o README;
2. leia toda a documentação atual do Engineering Core;
3. leia especialmente v0.4, v0.5 e v0.6;
4. examine a estrutura atual de ModelComponent;
5. examine SQLite, repositories e services existentes;
6. examine AI Tool Registry e Context Builder;
7. preserve toda funcionalidade já validada;
8. rode lint, testes e build antes de iniciar.

Não refatore módulos estáveis sem necessidade.

---

# Arquitetura conceitual

Criar uma nova camada:

```text
MODEL CORE
   ↓
COMPONENT CORE
   ↓
ASSEMBLY INTELLIGENCE
   ↓
AI ENGINEERING CONTEXT
```

Assembly Intelligence deve ficar separado da geometria Three.js.

Three.js representa o objeto.

Assembly Intelligence representa o significado.

---

# Persistência

A semântica deve sobreviver ao reinício do Azriel.

Criar migrations novas.

Não alterar migrations antigas já aplicadas.

---

# Component Semantic Metadata

Criar persistência conceitual equivalente a:

```ts
interface ComponentSemantic {
  id: string;

  modelIdentity: string;
  componentIdentity: string;

  semanticLabel?: string;
  subsystemId?: string;

  role?: string;
  description?: string;

  notes?: string;

  createdAt: string;
  updatedAt: string;
}
```

Adaptar à arquitetura real do projeto.

---

# Identidade do modelo

É necessário evitar aplicar metadata de um modelo em outro modelo diferente.

Criar uma estratégia de identificação estável.

Pode considerar:

* nome do arquivo;
* fingerprint/hash;
* estrutura do modelo;
* tamanho;
* metadata disponível.

Preferir hash/fingerprint quando viável.

Não depender apenas do nome do arquivo.

---

# Identidade do componente

O ID temporário criado durante a sessão não é suficiente para persistência.

Criar uma estratégia estável baseada em elementos como:

```text
node path
+
original name
+
hierarchy
```

Exemplo:

```text
Scene/MotorAssembly/Rotor
```

Se necessário, complementar com índice estrutural.

Documentar limitações.

---

# Rebinding

Ao carregar novamente o mesmo modelo:

o Assembly Intelligence deve tentar associar metadata persistida aos componentes atuais.

Se houver correspondência segura:

aplicar automaticamente.

Se houver ambiguidade:

não aplicar silenciosamente.

Mostrar estado:

```text
SEMANTIC REBIND
REVIEW REQUIRED
```

---

# Semantic Label

Permitir ao operador renomear semanticamente um componente.

Exemplo:

```text
ORIGINAL NAME
Mesh_047

SEMANTIC LABEL
Rotor
```

O nome original nunca deve ser destruído.

Manter:

```text
originalName
semanticLabel
```

---

# Exibição

Quando houver semantic label:

mostrar preferencialmente:

```text
Rotor
```

e secundariamente:

```text
Mesh_047
```

Exemplo:

```text
Rotor
Mesh_047
```

---

# Manual Classification

Criar no Component Inspector campos editáveis para:

```text
SEMANTIC LABEL
SUBSYSTEM
ROLE
DESCRIPTION
NOTES
```

Essas alterações são feitas manualmente pelo operador.

---

# Subsystems

Criar entidade persistente:

```ts
interface EngineeringSubsystem {
  id: string;
  modelIdentity: string;

  name: string;
  description?: string;

  parentSubsystemId?: string;

  createdAt: string;
  updatedAt: string;
}
```

---

# Exemplo

```text
Vehicle
│
├── Powertrain
│   ├── Engine
│   └── Transmission
│
├── Electrical
└── Suspension
```

ou:

```text
Motor
│
├── Rotating Assembly
├── Housing
└── Electrical
```

---

# Relação componente → subsistema

Um componente pode inicialmente pertencer a:

```text
1 subsystem principal
```

Não implementar múltiplos subsistemas por componente nesta versão, salvo se a arquitetura já tornar isso trivial.

Priorizar clareza.

---

# Subsystem Manager

Criar interface para:

* criar subsistema;
* editar;
* excluir;
* criar hierarquia;
* associar componentes;
* remover associação.

Excluir subsistema não deve excluir componentes.

---

# Roles

Criar campo textual controlado:

```text
role
```

Exemplos:

```text
rotating component
housing
fastener
sensor
bearing
electrical connector
structural support
```

Não obrigar taxonomia fechada nesta versão.

Mas permitir sugestões/autocomplete local se útil.

---

# Não inferir automaticamente

Se o modelo contém:

```text
Mesh_047
```

o Azriel NÃO pode definir:

```text
role = Rotor
```

automaticamente.

A classificação é explícita.

---

# Component Relationships

Criar relações persistentes entre componentes.

Estrutura conceitual:

```ts
interface ComponentRelationship {
  id: string;

  modelIdentity: string;

  sourceComponentId: string;
  targetComponentId: string;

  type: string;
  description?: string;

  createdAt: string;
}
```

---

# Relationship Types

Criar alguns tipos iniciais:

```text
connected_to
contains
supports
drives
mounted_on
adjacent_to
depends_on
custom
```

Esses tipos são semânticos.

Não afirmar relação geométrica automática como relação mecânica.

---

# Relações manuais

O operador deve conseguir criar:

```text
Rotor
drives
Shaft
```

ou:

```text
Bearing
supports
Shaft
```

---

# Relações geométricas

O sistema pode calcular separadamente informações como:

* proximidade;
* parent/child;
* bounding box overlap.

Mas isso deve ser chamado de:

```text
GEOMETRIC RELATIONSHIP
```

e não de relação mecânica.

Não confundir:

```text
adjacent geometrically
```

com:

```text
mechanically connected
```

---

# Assembly Graph

Criar uma visualização de grafo.

Representar:

* componentes;
* subsistemas;
* relações.

Exemplo:

```text
        Motor
          │
    ┌─────┴─────┐
    │           │
  Rotor       Housing
    │
  drives
    │
  Shaft
    │
 supported by
    │
 Bearing
```

Não precisa ser grafo 3D.

Pode ser SVG/2D.

---

# Graph Interaction

Ao clicar em um node:

selecionar o componente correspondente no Engineering View.

Sincronização:

```text
ASSEMBLY GRAPH
↕
COMPONENT TREE
↕
3D VIEW
↕
INSPECTOR
```

---

# Semantic Status

Adicionar estado por componente:

```text
UNCLASSIFIED
PARTIAL
CLASSIFIED
```

Exemplo:

UNCLASSIFIED:
nenhum semantic label.

PARTIAL:
label definido, mas sem subsystem/role.

CLASSIFIED:
campos essenciais definidos.

Definir regra simples e documentada.

---

# Coverage

Mostrar métricas:

```text
ASSEMBLY SEMANTICS

COMPONENTS        86
CLASSIFIED        41
PARTIAL           12
UNCLASSIFIED      33

SEMANTIC COVERAGE 61%
```

Não misturar isso com Mapa Stark.

É cobertura semântica do modelo atual.

---

# Filter

Permitir filtrar:

```text
ALL
CLASSIFIED
PARTIAL
UNCLASSIFIED
```

Isso ajuda a ensinar modelos grandes.

---

# Bulk Classification

Não implementar automação inteligente em massa.

Mas pode permitir ações simples de UI:

* selecionar vários componentes pela árvore;
* atribuir mesmo subsystem.

Somente se isso não aumentar muito o escopo.

Não obrigatório.

---

# Notes

Cada componente pode ter notas técnicas.

Exemplo:

```text
Rotor

NOTES
- verificar geometria do eixo
- provável interface com bearing frontal
```

Notas são texto do operador.

---

# Persistência de notas

Salvar no SQLite.

---

# Model Notes

Adicionar também notas para o assembly/modelo inteiro.

Exemplo:

```text
MODEL NOTES
```

Não obrigatório se arquitetura já tiver mecanismo equivalente.

---

# AI Core — READ

Expandir Engineering Tools com:

```text
get_component_semantics
get_subsystems
get_subsystem_components
get_component_relationships
get_unclassified_components
get_semantic_coverage
get_assembly_graph_summary
```

---

# AI Core — Context

O AI Core passa a poder responder:

```text
Quais componentes pertencem ao motor?
```

```text
O que está classificado como parte do conjunto rotativo?
```

```text
Qual componente está conectado ao eixo?
```

A resposta deve vir de metadata persistida.

---

# AI Semantic Safety

Se uma relação não estiver cadastrada:

não inventar.

Exemplo:

Pergunta:

```text
O rotor está conectado ao eixo?
```

Se não houver relação registrada:

responder:

```text
Não existe uma relação mecânica registrada entre esses componentes.
```

Pode mencionar hierarquia geométrica separadamente se houver.

---

# AI Core — Write

Nesta versão, não permitir que o LLM altere automaticamente semantic labels ou relações.

READ ONLY para Assembly Intelligence.

O operador ensina via UI.

---

# Sugestão por IA

Opcionalmente, o AI Core pode sugerir:

```text
O componente "Rotor" ainda não possui role definido.
Deseja classificá-lo?
```

Mas não executar a alteração.

---

# AI Queries obrigatórias

Suportar:

```text
Quais subsistemas existem?
```

```text
Quais componentes pertencem ao subsistema Motor?
```

```text
Qual é a função registrada do Rotor?
```

```text
Quais componentes ainda não foram classificados?
```

```text
Qual é a cobertura semântica deste modelo?
```

```text
Quais relações estão registradas para o Shaft?
```

```text
Esse componente possui notas?
```

---

# Contextual Reference

Se componente estiver selecionado:

```text
A que subsistema essa peça pertence?
```

usar selectedComponentId.

---

# Semantic Inspector

Expandir Component Inspector:

```text
IDENTIDADE
Rotor
Mesh_047

SUBSYSTEM
Rotating Assembly

ROLE
Rotating Component

DESCRIPTION
...

RELATIONSHIPS
→ drives Shaft
← supported_by Bearing

NOTES
...
```

---

# Relationship Editor

Adicionar UI simples:

```text
SOURCE
Rotor

RELATION
drives

TARGET
Shaft

[ ADICIONAR ]
```

---

# Validation

Não permitir:

* relação componente consigo mesmo;
* IDs inexistentes;
* componente de outro modelo;
* relation type vazio.

---

# Duplicate Relationship

Evitar duplicata idêntica.

---

# Relationship Direction

Tipos podem ser direcionais.

Exemplo:

```text
Rotor drives Shaft
```

não equivale automaticamente a:

```text
Shaft drives Rotor
```

---

# Inverse Labels

Opcionalmente a UI pode mostrar relação inversa de leitura:

```text
Shaft
driven by Rotor
```

Mas não é obrigatório persistir duas relações.

---

# Component Search

Busca existente deve considerar:

```text
originalName
semanticLabel
role
subsystem
```

---

# AI find_component

Atualizar busca para priorizar semantic labels.

Exemplo:

```text
"rotor"
```

deve encontrar componente mesmo que originalName seja:

```text
Mesh_047
```

---

# Exploded View Integration

Assembly semantic metadata não deve alterar algoritmo atual de explosão nesta versão.

Mas preparar interface para futuro uso de subsistemas.

Exemplo futuro:

```text
explode_subsystem("Motor")
```

Não implementar ainda, salvo se trivial.

---

# Selection

Selecionar componente pelo Assembly Graph deve manter compatibilidade com:

* isolate;
* focus;
* exploded view;
* AI Core;
* gestures.

---

# Data separation

Não guardar semântica diretamente dentro de userData do Three.js como única fonte.

Three.js pode receber referência temporária.

SQLite é a fonte persistente.

---

# Import / Export

Criar exportação simples da camada semântica para JSON.

Exemplo:

```json
{
  "model": "...",
  "subsystems": [],
  "components": [],
  "relationships": []
}
```

Isso serve para:

* backup;
* portabilidade;
* inspeção.

---

# Import

Se seguro e simples, permitir importar metadata JSON compatível.

Validar modelIdentity.

Se não corresponder:

não aplicar sem aviso.

Import é desejável, mas pode ser opcional se aumentar muito o escopo.

Export é obrigatório.

---

# Arquivo 3D

Nunca modificar GLB/GLTF original.

Semantic metadata fica separada.

---

# UI

Adicionar ao Engineering Core abas ou áreas equivalentes:

```text
MODEL
COMPONENTS
ASSEMBLY
SEMANTICS
```

Não transformar a interface em sistema corporativo.

Preservar HUD técnico atual.

---

# Assembly Intelligence Status

HUD:

```text
ASSEMBLY INTELLIGENCE
ONLINE

SUBSYSTEMS
05

RELATIONSHIPS
17

SEMANTIC COVERAGE
61%
```

---

# Estados existentes

Manter:

```text
MODEL CORE
ONLINE

COMPONENT CORE
ONLINE

EXPLOSION CORE
ONLINE

AI INTEGRATION
ONLINE
```

---

# Digital Twin

Continuar:

```text
DIGITAL TWIN
NÃO CONECTADO
```

---

# IoT

Continuar:

```text
IOT
NÃO CONECTADO
```

---

# Testes

Adicionar testes para:

1. model identity;
2. component persistent identity;
3. semantic label;
4. original name preservation;
5. subsystem creation;
6. subsystem hierarchy;
7. component assignment;
8. relationship creation;
9. relationship direction;
10. duplicate prevention;
11. self relationship rejection;
12. model mismatch;
13. semantic coverage;
14. unclassified filtering;
15. persistence;
16. reload/rebind;
17. ambiguous rebind;
18. export JSON;
19. AI read tools;
20. selected component context;
21. search by semantic label;
22. search by subsystem;
23. no hallucinated relationship.

---

# Persistência obrigatória

Teste:

1. carregar modelo;
2. selecionar `Mesh_047`;
3. definir semantic label `Rotor`;
4. atribuir subsystem `Motor`;
5. definir role;
6. criar relação com outro componente;
7. fechar Azriel;
8. abrir novamente;
9. carregar o mesmo modelo;
10. confirmar que metadata foi restaurada corretamente.

---

# Teste com mudança de modelo

Carregar modelo diferente.

Metadata do primeiro modelo não pode aparecer.

---

# Documentação

Criar:

```text
docs/engineering-core/v0.7.md
```

Documentar:

* objetivo;
* Assembly Intelligence;
* model identity;
* persistent component identity;
* semantic labels;
* subsystems;
* relationships;
* graph;
* coverage;
* AI tools;
* safety;
* export;
* limitations.

Registrar:

```text
v0.1 — Hand Interaction — concluída
v0.2 — Spatial Manipulation — concluída
v0.2.1 — Calibration Refinement — concluída
v0.3 — Model Core — concluída
v0.4 — Component Core — concluída
v0.5 — Exploded View — concluída
v0.6 — AI Integration — concluída
v0.7 — Assembly Intelligence
```

---

# Critérios de aceite

Engineering Core v0.7 só está concluído quando:

1. Assembly Intelligence existir;
2. metadata semântica for persistida;
3. nome original for preservado;
4. semantic label puder ser definido;
5. subsystem puder ser criado;
6. subsystem puder possuir hierarquia;
7. componente puder ser associado;
8. role puder ser definido;
9. descrição puder ser definida;
10. notes puderem ser registradas;
11. relações puderem ser criadas;
12. relações forem direcionais;
13. duplicatas forem evitadas;
14. relação consigo mesmo for bloqueada;
15. Assembly Graph existir;
16. grafo sincronizar com seleção 3D;
17. semantic coverage for calculada;
18. filtros classificados/não classificados funcionarem;
19. metadata sobreviver ao reinício;
20. mesmo modelo recuperar metadata;
21. modelo diferente não receber metadata incorreta;
22. ambiguous rebind for tratado;
23. busca considerar semantic labels;
24. AI Core consultar semântica;
25. AI Core consultar subsistemas;
26. AI Core consultar relações;
27. AI Core não inventar relações;
28. AI Core não editar semântica;
29. export JSON funcionar;
30. GLB original permanecer intocado;
31. exploded view continuar funcionando;
32. gestures continuarem funcionando;
33. AI Integration continuar funcionando;
34. TypeScript não apresentar erros;
35. testes passarem;
36. build funcionar;
37. Tauri iniciar normalmente;
38. demais módulos do Azriel permanecerem funcionando.

---

# Teste final obrigatório

Carregar um modelo com múltiplos componentes.

Selecionar um componente com nome ruim:

```text
Mesh_047
```

Definir:

```text
SEMANTIC LABEL
Rotor
```

Criar:

```text
SUBSYSTEM
Motor
```

Associar Rotor ao Motor.

Definir:

```text
ROLE
Rotating Component
```

Selecionar outro componente e classificar:

```text
Shaft
```

Criar relação:

```text
Rotor
drives
Shaft
```

Confirmar no Assembly Graph.

Selecionar Rotor pelo grafo.

Confirmar seleção no 3D.

Perguntar ao Azriel:

```text
A que subsistema o Rotor pertence?
```

Resposta deve utilizar dados persistidos.

Perguntar:

```text
Qual relação existe entre Rotor e Shaft?
```

Resposta:

```text
Rotor drives Shaft
```

Perguntar sobre relação não cadastrada.

Azriel deve informar que não existe relação registrada.

Exportar metadata para JSON.

Fechar Azriel.

Abrir novamente.

Carregar o mesmo modelo.

Confirmar:

```text
Mesh_047
→ Rotor
→ Motor
→ Rotating Component
```

e relação:

```text
Rotor → drives → Shaft
```

restauradas corretamente.

---

# Resultado esperado

Ao concluir a v0.7, o Azriel deixa de enxergar apenas:

```text
Mesh_047
Mesh_048
Mesh_049
```

e passa a possuir uma camada persistente como:

```text
MOTOR
│
├── Rotor
│    role: Rotating Component
│
├── Shaft
│
└── Bearing

Rotor
   └── drives → Shaft
```

A geometria continua vindo do modelo 3D.

O significado vem de conhecimento explicitamente registrado.

Isso permite que o AI Core trabalhe sobre uma representação confiável da montagem.

A filosofia da versão é:

# GEOMETRIA MOSTRA A FORMA. SEMÂNTICA EXPLICA O QUE ELA REPRESENTA.
