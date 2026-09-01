# AZRIEL — Engineering Core v0.4 — Component Core

A versão **Engineering Core v0.3 — Model Core** foi concluída e validada.

O Engineering Core já consegue:

* carregar modelos GLB/GLTF;
* enquadrar automaticamente;
* inspecionar metadata;
* visualizar hierarquia;
* mover;
* rotacionar;
* escalar;
* utilizar hand tracking;
* utilizar wireframe/grid/axes;
* descarregar modelos corretamente.

Agora implemente:

# Engineering Core v0.4 — Component Core

---

# Antes de modificar código

1. leia o README;
2. leia `docs/engineering-core/v0.1.md`;
3. leia `docs/engineering-core/v0.2.md`;
4. leia `docs/engineering-core/v0.3.md`;
5. examine a arquitetura atual do Model Core;
6. preserve MOVE / ROTATE / SCALE;
7. preserve hand tracking;
8. preserve ModelRoot;
9. preserve carregamento GLB/GLTF;
10. rode lint, testes e build antes de iniciar.

Não redesenhe o Engineering View sem necessidade.

---

# Objetivo

Transformar modelos 3D carregados em estruturas inspecionáveis por componente.

Ao final desta versão, o operador deve conseguir:

* visualizar a hierarquia do modelo;
* apontar para uma peça;
* identificar a mesh sob o cursor;
* destacar uma peça;
* selecionar uma peça;
* inspecionar suas propriedades;
* ocultar uma peça;
* isolar uma peça;
* restaurar o modelo;
* selecionar componentes através da árvore do modelo.

O modelo deixa de ser tratado apenas como um objeto único.

---

# Princípio central

A v0.4 deve provar:

# O OPERADOR CONSEGUE IDENTIFICAR E MANIPULAR COMPONENTES INDIVIDUAIS DE UM MODELO 3D.

Isso prepara diretamente:

```text id="jly53b"
v0.5 → Exploded View
```

---

# Fora do escopo

Não implementar ainda:

* exploded view;
* física;
* constraints mecânicos;
* CAD nativo;
* edição de geometria;
* Digital Twin;
* IoT;
* telemetria;
* AI Core controlando componentes;
* voz;
* alteração do arquivo original.

---

# Component Model

Criar uma representação interna dos componentes encontrados no GLTF.

Exemplo conceitual:

```ts id="vnwbeq"
interface ModelComponent {
  id: string;
  name: string;
  type: string;

  parentId?: string;
  children: string[];

  visible: boolean;
  selectable: boolean;

  meshCount: number;

  originalPosition: Vector3;
  originalRotation: Euler;
  originalScale: Vector3;
}
```

Adaptar aos tipos existentes.

Não armazenar objetos Three.js diretamente no SQLite.

O Component Core pode manter referências em memória durante a sessão.

---

# Identificação

Ao carregar um modelo:

percorrer sua scene graph.

Identificar:

* Groups;
* Meshes;
* nested Groups;
* nomes;
* hierarquia.

Criar IDs internos estáveis durante a sessão.

---

# Nomes

Preservar nomes definidos no GLB/GLTF quando existirem.

Exemplo:

```text id="u3e2v0"
MotorAssembly
Rotor
Housing
Shaft
Connector
```

Se um node não possuir nome:

gerar algo previsível:

```text id="95s9xu"
Mesh_001
Mesh_002
Group_003
```

Não usar nomes aleatórios.

---

# Hierarquia

O painel existente de estrutura deve evoluir para uma árvore interativa.

Exemplo:

```text id="u3hskt"
MODEL STRUCTURE

▼ MotorAssembly
   ├─ Housing
   ├─ Rotor
   │   └─ Shaft
   ├─ Bearing
   └─ Connector
```

Permitir:

* expandir;
* recolher;
* selecionar.

---

# Seleção pela árvore

Ao clicar em:

```text id="p2ek24"
Rotor
```

o componente correspondente deve:

* ser selecionado;
* receber highlight;
* aparecer no Component Inspector.

---

# Seleção espacial

Implementar seleção diretamente no viewport.

Utilizar:

```text id="72mshs"
Raycaster
```

ou mecanismo equivalente do React Three Fiber.

Fluxo:

```text id="fpb28n"
cursor
↓
raycast
↓
mesh encontrada
↓
resolver componente
↓
TARGETED
```

---

# Hand Cursor

Integrar com o cursor virtual já existente.

Quando POINT ou cursor estiver sobre uma mesh:

```text id="mujesf"
COMPONENT
TARGETED
```

Mostrar highlight temporário.

---

# Mouse

Seleção deve funcionar também com mouse.

Isso é importante para:

* desenvolvimento;
* depuração;
* uso sem webcam.

---

# Estados de componente

Criar:

```text id="qf3qds"
normal
targeted
selected
hidden
isolated
```

Não permitir estados contraditórios.

---

# Targeted

Quando cursor estiver sobre componente:

mostrar highlight discreto.

Quando cursor sair:

remover highlight, exceto se estiver selecionado.

---

# Selected

Ao selecionar:

manter highlight persistente.

Atualizar inspector.

Exemplo:

```text id="wz3c0l"
SELECTED COMPONENT

Rotor
```

---

# Seleção por gesto

Utilizar inicialmente:

```text id="56pt8d"
POINT
```

para target.

E:

```text id="5b7fp7"
PINCH
```

para selecionar.

Fluxo:

```text id="w4uc7p"
POINT
↓
Rotor TARGETED
↓
PINCH
↓
Rotor SELECTED
```

Não quebrar MOVE / ROTATE / SCALE.

---

# Modo de interação

Adicionar:

```text id="bdhdxc"
MODE

MODEL
COMPONENT
```

## MODEL

Mantém comportamento atual:

* MOVE;
* ROTATE;
* SCALE do modelo inteiro.

## COMPONENT

Permite:

* target;
* select;
* inspect;
* hide;
* isolate.

Isso evita conflitos entre manipulação do modelo e seleção de peças.

---

# Component Inspector

Criar painel de informações.

Exemplo:

```text id="3t5cpk"
COMPONENT

NAME
Rotor

TYPE
Mesh

PARENT
MotorAssembly

CHILDREN
2

MESHES
1

VISIBLE
YES
```

Adicionar transformações locais:

```text id="2rhjsd"
POSITION
X
Y
Z

ROTATION
X
Y
Z

SCALE
X
Y
Z
```

---

# Geometry Stats

Se puder ser obtido de forma eficiente:

mostrar:

```text id="vd0o4i"
VERTICES
TRIANGLES
```

Não recalcular a cada frame.

---

# Material Inspector

Mostrar informações básicas quando disponíveis:

```text id="d3g88o"
MATERIAL
Steel

TYPE
MeshStandardMaterial

COLOR
#...

TEXTURE
YES / NO
```

Não criar editor de material.

Somente leitura.

---

# Bounding Box

Calcular bounding box do componente selecionado.

Opcionalmente mostrar visualmente uma bounding box técnica.

Adicionar:

```text id="8j1md3"
BOUNDING BOX
ON / OFF
```

---

# Highlight

Não destruir materiais originais para destacar componentes.

Criar mecanismo reversível.

Possibilidades:

* emissive temporário;
* outline;
* overlay;
* material clone controlado.

Escolher solução compatível com os materiais existentes.

Após remover highlight:

material original deve permanecer intacto.

---

# Hide Component

Adicionar ação:

```text id="f4c2wu"
OCULTAR COMPONENTE
```

Resultado:

```text id="hmd8cp"
visible = false
```

Atualizar árvore.

Exemplo:

```text id="6v87c7"
Rotor
HIDDEN
```

---

# Show Component

Para componente oculto:

```text id="ok0rm5"
MOSTRAR COMPONENTE
```

Restaurar visibilidade.

---

# Isolate Component

Adicionar:

```text id="tzl4pm"
ISOLAR COMPONENTE
```

Resultado:

* componente selecionado permanece visível;
* demais componentes relevantes ficam ocultos.

Mostrar:

```text id="fd6v4s"
ISOLATION MODE
ACTIVE
```

---

# Exit Isolation

Adicionar:

```text id="43qmev"
SAIR DO ISOLAMENTO
```

Restaurar visibilidade anterior.

Não assumir que todos os componentes estavam originalmente visíveis.

Guardar estado anterior.

---

# Restore Model

Adicionar:

```text id="c4ok2n"
RESTAURAR COMPONENTES
```

Restaurar:

* visibilidade;
* seleção;
* isolamento;
* highlights.

Não alterar MOVE / ROTATE / SCALE global.

---

# Multi-selection

Não implementar seleção múltipla nesta versão.

Apenas:

```text id="3r6npu"
selectedComponentId
```

Uma peça por vez.

---

# Component Search

Adicionar busca simples:

```text id="6ynv9p"
BUSCAR COMPONENTE
```

Pesquisar por nome.

Ao selecionar resultado:

* expandir árvore;
* selecionar componente;
* enquadrar quando apropriado.

---

# Focus Component

Adicionar:

```text id="i79i22"
FOCAR
```

Ajustar câmera para enquadrar componente selecionado.

Não alterar transformações do modelo.

---

# Return to Model

Adicionar:

```text id="udyrgj"
FOCAR MODELO
```

Restaurar framing do modelo inteiro.

---

# Modelo sem componentes úteis

Alguns GLBs podem conter:

```text id="60fc6d"
Scene
└── Mesh
```

Nesse caso:

não inventar uma montagem complexa.

Mostrar simplesmente a estrutura existente.

Component Core ainda deve funcionar.

---

# Modelo com mesh única

Se existir apenas uma mesh:

seleção funciona normalmente.

Mostrar:

```text id="wue1zh"
COMPONENTS
1
```

Exploded View futuro terá funcionalidade limitada nesse tipo de asset.

---

# Preparação para Exploded View

Guardar para cada componente:

```text id="w63mza"
originalPosition
originalRotation
originalScale
```

Também calcular:

```text id="7ob4kp"
worldPosition
modelCenter
directionFromCenter
```

Não executar explosão ainda.

Esses dados serão utilizados na v0.5.

---

# Direction From Center

Para cada componente relevante:

```text id="qnj6iw"
direction =
componentCenter - modelCenter
```

Normalizar quando apropriado.

Não mover componente nesta versão.

Apenas preparar metadata.

---

# Component Hierarchy Service

Criar camada separada.

Exemplo:

```text id="b78y0c"
ComponentService
```

Responsável por:

* mapear scene graph;
* encontrar componente;
* buscar por nome;
* resolver mesh → component;
* calcular metadata;
* gerenciar visibilidade;
* isolamento.

Não colocar tudo dentro do React component.

---

# Performance

Scene traversal deve ocorrer principalmente:

```text id="4u0jcn"
on model load
```

Não percorrer toda a árvore a cada frame.

Raycasting deve considerar apenas objetos selecionáveis.

---

# Model Root

Preservar arquitetura:

```text id="mgdzpm"
ModelRoot
   │
   └── Components
```

Transformações globais continuam em `ModelRoot`.

Transformações internas pertencem aos nodes.

---

# Segurança

Component Core é somente visual/manipulação em memória.

Não:

* alterar arquivo GLB;
* salvar modificações;
* sobrescrever asset;
* executar scripts;
* enviar modelo ao Ollama.

---

# AI Core

Não integrar ainda.

Mas preparar interfaces futuras de leitura:

```text id="0vgm8p"
get_model_components
get_selected_component
get_component_details
```

E futuras ações controladas:

```text id="g1s02r"
select_component
focus_component
```

Não registrar como AI tools nesta versão.

---

# HUD

Atualizar Engineering View.

Quando modelo estiver carregado:

```text id="7x92lv"
MODEL CORE
ONLINE

COMPONENT CORE
ONLINE

COMPONENTS
24

SELECTED
Rotor
```

Quando nada selecionado:

```text id="m01dl8"
SELECTED
NONE
```

---

# Estados existentes

Manter:

```text id="txg7jy"
CAD
NÃO CONECTADO

DIGITAL TWIN
NÃO CONECTADO

IOT
NÃO CONECTADO
```

Component Core não significa CAD nativo.

---

# Testes

Adicionar testes para:

1. scene traversal;
2. component IDs;
3. nomes existentes;
4. fallback names;
5. parent/children;
6. mesh → component;
7. search;
8. selection;
9. targeted;
10. highlight restore;
11. hide;
12. show;
13. isolate;
14. exit isolation;
15. previous visibility state;
16. restore components;
17. component bounding box;
18. component center;
19. direction from model center;
20. model with one mesh;
21. nested groups.

Utilizar fixtures simples.

---

# Documentação

Criar:

```text id="q3pzk6"
docs/engineering-core/v0.4.md
```

Documentar:

* objetivo;
* Component Core;
* scene graph;
* component mapping;
* raycasting;
* seleção;
* highlight;
* inspector;
* hide/show;
* isolation;
* hierarchy;
* preparação para exploded view;
* limitações.

Registrar:

```text id="f2zqvu"
v0.1 — Hand Interaction — concluída
v0.2 — Spatial Manipulation — concluída
v0.3 — Model Core — concluída
v0.4 — Component Core
```

Manter registrado:

```text id="4pzj8j"
Known limitation:
calibração gestual ainda requer refinamento.
```

---

# Critérios de aceite

Engineering Core v0.4 só está concluído quando:

1. modelo carregar normalmente;
2. scene graph for processada;
3. componentes forem identificados;
4. hierarquia aparecer;
5. árvore for interativa;
6. seleção pela árvore funcionar;
7. raycasting funcionar;
8. seleção por mouse funcionar;
9. target por hand cursor funcionar;
10. pinch selecionar componente;
11. MODEL mode continuar funcionando;
12. COMPONENT mode existir;
13. highlight targeted funcionar;
14. highlight selected funcionar;
15. materiais originais forem preservados;
16. inspector mostrar informações reais;
17. transform local aparecer;
18. hide funcionar;
19. show funcionar;
20. isolate funcionar;
21. exit isolation funcionar;
22. visibilidade anterior for restaurada;
23. restore components funcionar;
24. busca funcionar;
25. focus component funcionar;
26. focus model funcionar;
27. bounding box funcionar quando habilitada;
28. componente selecionado permanecer consistente;
29. modelo com mesh única funcionar;
30. metadata para exploded view for preparada;
31. MOVE / ROTATE / SCALE global continuarem funcionando;
32. hand tracking continuar funcionando;
33. TypeScript não apresentar erros;
34. testes passarem;
35. build funcionar;
36. Tauri iniciar normalmente;
37. demais módulos do Azriel não sofrerem regressão.

---

# Teste final obrigatório

Carregar um modelo GLB com múltiplas meshes/nodes.

Entrar em:

```text id="cqwc1o"
MODE = COMPONENT
```

Passar o mouse sobre uma peça.

Confirmar:

```text id="ej4j1w"
TARGETED
```

Clicar.

Confirmar:

```text id="0p7zjj"
SELECTED
```

Verificar Component Inspector.

Selecionar outra peça pela árvore.

Confirmar sincronização entre:

```text id="dr3xyh"
TREE
↕
VIEWPORT
↕
INSPECTOR
```

Ativar webcam.

Apontar para uma peça.

Fazer pinch.

Confirmar seleção.

Executar:

```text id="h6a9aq"
OCULTAR COMPONENTE
```

Confirmar desaparecimento.

Executar:

```text id="0vt85c"
MOSTRAR COMPONENTE
```

Depois:

```text id="k8wp5c"
ISOLAR COMPONENTE
```

Confirmar que apenas a peça permanece.

Executar:

```text id="t3r3zw"
SAIR DO ISOLAMENTO
```

Confirmar restauração correta.

Buscar um componente pelo nome.

Executar:

```text id="o6a1dt"
FOCAR
```

Confirmar enquadramento.

Voltar:

```text id="h84pmz"
FOCAR MODELO
```

Finalmente trocar para:

```text id="cv8xv1"
MODE = MODEL
```

Confirmar que MOVE / ROTATE / SCALE continuam funcionando no modelo inteiro.

---

# Resultado esperado

Ao concluir a v0.4, o Engineering Core deve deixar de enxergar:

```text id="0qt8gt"
motor.glb
```

como apenas um objeto.

Ele deve começar a enxergar:

```text id="du3z8x"
MOTOR ASSEMBLY
│
├── Housing
├── Rotor
├── Shaft
├── Bearing
└── Connector
```

E o operador deve conseguir apontar para uma dessas partes, selecioná-la e inspecioná-la.

Essa é a fundação necessária para a próxima etapa:

# Engineering Core v0.5 — Exploded View

Na v0.5, os componentes que agora conseguimos identificar finalmente poderão ser separados espacialmente e depois reconstruídos.

---

# Estado da implementação — 01/09/2026

Status: **concluída e validada em 01/09/2026**.

Foram entregues o `ComponentService`, o mapeamento determinístico da scene graph, a árvore interativa com busca, seleção sincronizada por árvore/mouse/mão, inspector técnico, highlights reversíveis, bounding box, hide/show, isolamento com restauração da visibilidade anterior e foco de componente/modelo.

Os modos `MODEL` e `COMPONENT` mantêm responsabilidades separadas. MOVE, ROTATE e SCALE continuam atuando no ModelRoot; target e seleção de peças operam somente no modo COMPONENT. Nenhum material ou arquivo GLB/GLTF é alterado.

A suíte automatizada cobre o Component Core com fixtures locais. O fluxo entregue foi aprovado pelo operador e a v0.4 está encerrada.

Known limitation: a calibração gestual ainda requer refinamento e permanece planejada para v0.2.1.

## Complemento corretivo — manipulação gestual de componentes

Após a primeira validação foi identificada uma lacuna: o modo COMPONENT selecionava e inspecionava peças, mas MOVE, ROTATE e SCALE ainda atuavam somente no ModelRoot.

Foi criado um controlador independente para transformações locais. Agora uma peça selecionada pode ser movida ou rotacionada por pinça e escalada com duas mãos, sem alterar o ModelRoot. A primeira pinça seleciona; após o release, a próxima pinça inicia a transformação. Perda de tracking encerra a sessão sem desfazer a última posição.

O inspector passou a exibir transformações atuais e originais. `RESTAURAR COMPONENTES` também restaura posição, rotação e escala locais. O complemento possui testes de MOVE relativo, ROTATE, SCALE proporcional, troca de componente, release, perda de tracking e restauração.

Status do complemento: **concluído e revalidado pelo operador em 01/09/2026**.

Com a aprovação de MOVE, ROTATE e SCALE sobre o componente selecionado, a v0.4 está definitivamente encerrada.
