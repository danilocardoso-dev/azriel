# AZRIEL — Engineering Core v0.5 — Exploded View

A versão **Engineering Core v0.4 — Component Core** foi concluída e validada.

O Engineering Core já consegue:

* detectar mãos;
* reconhecer gestos;
* MOVE / ROTATE / SCALE;
* carregar modelos GLB/GLTF;
* interpretar a scene graph;
* identificar componentes;
* selecionar componentes por viewport, árvore e gestos;
* destacar componentes;
* ocultar e mostrar;
* isolar;
* inspecionar;
* calcular centros e bounding boxes;
* preservar transformações originais.

Agora implemente:

# Engineering Core v0.5 — Exploded View

---

# Objetivo

Permitir desmontar visualmente uma montagem 3D em seus componentes e reconstruí-la de forma contínua e determinística.

Ao final desta versão, o operador deve conseguir:

* explodir uma montagem;
* controlar continuamente a distância da explosão;
* reconstruir a montagem;
* selecionar componentes durante a explosão;
* explodir apenas grupos/componentes selecionados quando possível;
* controlar a explosão pela interface;
* controlar a explosão usando duas mãos;
* visualizar linhas técnicas relacionando peças às posições originais.

A montagem original nunca deve ser modificada permanentemente.

---

# Princípio central

A v0.5 deve provar:

# UMA MONTAGEM PODE SER ABERTA E RECONSTRUÍDA ESPACIALMENTE.

O comportamento deve ser:

* previsível;
* reversível;
* contínuo;
* visualmente compreensível.

---

# Fora do escopo

Não implementar:

* física real;
* gravidade;
* colisões;
* joints mecânicos;
* constraints CAD;
* desmontagem fisicamente correta baseada em engenharia;
* STEP;
* IGES;
* edição de geometria;
* salvamento do GLB modificado;
* Digital Twin;
* IoT;
* telemetria;
* comandos do AI Core;
* voz.

---

# Estado original

Antes de qualquer explosão, preservar para cada componente:

```ts id="p1wr6o"
interface OriginalTransform {
  position: Vector3;
  rotation: Euler;
  scale: Vector3;
}
```

Se isso já existir na v0.4, reutilizar.

Não duplicar estado.

---

# Explosion State

Criar estrutura conceitual:

```ts id="55grlz"
interface ExplosionState {
  enabled: boolean;
  factor: number;
  mode: "all" | "selected";
  selectedRootId?: string;
}
```

`factor` deve variar entre:

```text id="ezkl94"
0.0 → montagem original
1.0 → explosão máxima configurada
```

Internamente pode haver multiplicador adicional de distância.

---

# Regra fundamental

A posição explodida deve SEMPRE ser derivada da posição original.

Não fazer:

```text id="yxtn22"
currentPosition += offset
```

a cada frame.

Isso causaria drift.

Fazer conceitualmente:

```text id="72f1b5"
explodedPosition =
originalPosition
+
explosionDirection
*
explosionDistance
*
explosionFactor
```

Assim:

```text id="pm5v6o"
factor = 0
```

sempre retorna exatamente à montagem original.

---

# Explosion Metadata

Criar metadata por componente:

```ts id="j92x51"
interface ExplosionMetadata {
  componentId: string;
  originalPosition: Vector3;
  worldCenter: Vector3;
  direction: Vector3;
  distanceMultiplier: number;
}
```

---

# Algoritmo inicial

Utilizar como primeira estratégia:

```text id="s3u1zm"
componentCenter
-
modelCenter
=
directionFromCenter
```

Normalizar a direção.

Depois aplicar distância baseada no tamanho do modelo.

---

# Problema do centro

Componentes muito próximos do centro podem gerar direção quase zero.

Criar fallback determinístico.

Pode considerar:

* eixo dominante;
* posição do parent;
* bounding box;
* índice espacial.

Não utilizar direção aleatória.

---

# Hierarquia

Exploded View deve respeitar scene graph.

Não tratar indiscriminadamente todos os meshes internos como peças independentes se eles pertencem a um componente lógico já identificado pela v0.4.

Utilizar a abstração `ModelComponent`.

---

# Explode All

Adicionar:

```text id="0twuvb"
EXPLODIR MONTAGEM
```

Ao ativar:

todos os componentes elegíveis recebem offsets.

Não mover `ModelRoot`.

Mover componentes relativos ao ModelRoot.

---

# Explosion Factor

Adicionar controle contínuo:

```text id="jkp8qv"
EXPLOSÃO

0% ─────────●──────── 100%
```

O slider deve atualizar em tempo real.

Exemplos:

```text id="2hd6rb"
0%
MONTADO

25%
PARCIAL

50%
ABERTO

100%
EXPLODIDO
```

---

# Reassembly

Ao reduzir o slider:

componentes devem retornar suavemente.

Quando:

```text id="bpgv1h"
factor = 0
```

todos devem estar exatamente nas posições originais.

Não aceitar erro acumulado perceptível.

---

# Reset Assembly

Adicionar:

```text id="2mwhgd"
RECONSTRUIR
```

Comportamento:

```text id="fj1pyz"
factor → 0
```

Pode animar suavemente.

Ao terminar:

```text id="nxctng"
ASSEMBLY
NOMINAL
```

---

# Animação

Adicionar transição opcional.

Estados:

```text id="0gnd9j"
assembled
exploding
exploded
reassembling
```

Utilizar interpolação.

Não utilizar física.

---

# Velocidade

Adicionar configuração simples:

```text id="65tfia"
EXPLOSION SPEED
```

ou valor interno razoável.

Não criar painel excessivamente complexo.

---

# Explode Selected

Adicionar modo:

```text id="q7ex0p"
EXPLOSÃO

ALL
SELECTED
```

Quando `SELECTED`:

utilizar o componente selecionado como raiz.

---

# Componente selecionado com filhos

Se componente selecionado possuir children:

explodir sua subárvore.

Exemplo:

```text id="wkyf30"
MotorAssembly
├── Rotor
├── Shaft
└── Bearing
```

Selecionar:

```text id="6j1dvy"
MotorAssembly
```

e usar:

```text id="u0g95u"
EXPLODE SELECTED
```

deve separar:

```text id="6om9ub"
Rotor
Shaft
Bearing
```

relativamente ao conjunto selecionado.

---

# Leaf Component

Se o componente selecionado não possuir filhos úteis:

não fingir que há montagem interna.

Mostrar:

```text id="n0yjkf"
SEM SUBCOMPONENTES PARA EXPLODIR
```

---

# Nested Explosion

Não implementar múltiplos níveis independentes simultaneamente nesta versão.

Apenas:

```text id="4sk21s"
ALL
```

ou:

```text id="p4qf2b"
SELECTED ROOT
```

---

# Selection During Explosion

Component Core deve continuar funcionando quando:

```text id="7yuhj9"
factor > 0
```

Operador deve conseguir:

* apontar;
* target;
* selecionar;
* inspecionar;
* ocultar;
* isolar;
* focar.

---

# Raycasting

Garantir que raycasting utilize posições atuais dos componentes explodidos.

Não usar bounding boxes antigas para seleção.

---

# Inspector

Adicionar seção:

```text id="q5j3li"
ASSEMBLY

STATE
EXPLODED

FACTOR
0.72

MODE
ALL
```

Para componente selecionado:

```text id="fqk3xi"
COMPONENT

Rotor

ORIGINAL POSITION
...

CURRENT POSITION
...

EXPLOSION OFFSET
...
```

---

# Technical Guide Lines

Adicionar opção:

```text id="r2wumg"
GUIDE LINES
ON / OFF
```

Quando ativa:

desenhar linha técnica entre:

```text id="0pjax4"
posição original
────────────
posição explodida
```

Isso deve ajudar a compreender de onde a peça veio.

---

# Guide Line Style

Utilizar estética do Azriel:

* linha fina;
* cyan discreto;
* transparência;
* sem glow exagerado.

Não transformar em efeito decorativo pesado.

---

# Origin Marker

Opcionalmente mostrar pequeno marcador na posição original.

Exemplo:

```text id="b04i7q"
+
```

ou retículo técnico.

---

# Connector Lines

Não afirmar que guide line representa:

* eixo mecânico;
* conexão elétrica;
* relação física.

Ela representa apenas:

# ORIGEM ESPACIAL DA PEÇA.

Documentar isso.

---

# Duas mãos

Integrar exploded view com tracking já existente.

Adicionar modo:

```text id="12fj7j"
CONTROL
EXPLODE
```

---

# Gesture Explosion

Quando:

* duas mãos detectadas;
* modo EXPLODE ativo;

usar distância relativa entre mãos.

---

# Session Start

Ao iniciar:

```text id="8hwxsn"
initialHandDistance
initialExplosionFactor
```

---

# Movimento

Quando mãos se afastarem:

```text id="1tue1q"
explosionFactor aumenta
```

Quando mãos se aproximarem:

```text id="u77z29"
explosionFactor diminui
```

Clamp:

```text id="gjzztv"
0.0
≤ factor ≤
1.0
```

---

# Exemplo

```text id="6s1zkd"
        🤲

MONTADO
factor 0.0


   ← 🤲      🤲 →

PARCIAL
factor 0.5


← 🤲              🤲 →

EXPLODIDO
factor 1.0
```

---

# Smoothing

Aplicar smoothing à distância entre mãos.

A calibração ainda é uma limitação conhecida.

Não tentar resolver toda calibração nesta versão.

---

# Loss of Tracking

Se uma mão desaparecer:

```text id="7hl30b"
GESTURE EXPLOSION
CANCELLED
```

Manter último factor válido.

Não fazer montagem saltar.

---

# Mouse / UI fallback

Exploded View deve funcionar perfeitamente sem webcam.

Slider é o controle principal confiável.

Gestos são uma segunda interface.

Isso facilita desenvolvimento e depuração.

---

# Model Manipulation

Durante exploded view:

MOVE / ROTATE / SCALE do `ModelRoot` devem continuar funcionando.

Exemplo:

modelo inteiro pode estar explodido e ainda ser rotacionado.

Offsets dos componentes permanecem relativos ao ModelRoot.

---

# Component Movement

Não permitir mover manualmente componentes individuais nesta versão.

Isso evitará conflito entre:

* explosion offset;
* transform original;
* manipulação manual.

Pode entrar futuramente.

---

# Restore after Hide/Isolation

Se usuário estiver em isolamento:

definir comportamento consistente antes de explodir.

Sugestão:

Exploded View deve respeitar visibilidade atual.

Não tornar componentes ocultos visíveis automaticamente.

---

# Exit Isolation

Ao sair:

restaurar estado esperado e aplicar explosion factor atual aos componentes restaurados.

---

# Model Change

Ao carregar outro modelo:

resetar:

```text id="f3of05"
explosionFactor = 0
mode = all
selectedRoot = null
```

Limpar metadata antiga.

---

# Unload

Ao descarregar modelo:

limpar Explosion State.

Não manter referências a components antigos.

---

# Explosion Service

Criar camada separada.

Sugestão:

```text id="wdx0v9"
ExplosionService
```

Responsável por:

* gerar metadata;
* calcular directions;
* calcular offsets;
* aplicar factor;
* reset;
* selected subtree;
* guide line data.

Não colocar algoritmo no componente React principal.

---

# Determinismo

Para o mesmo modelo:

a explosão deve gerar o mesmo resultado.

Não utilizar:

```text id="l35o61"
Math.random()
```

para posicionamento.

---

# Collision

Não implementar collision avoidance real.

Entretanto, componentes muito sobrepostos após explosão prejudicam a visualização.

Pode aplicar heurística simples baseada em:

* bounding boxes;
* direção;
* nível da hierarquia;
* distance multiplier.

Não criar solver físico.

---

# Hierarchy Depth

Pode utilizar profundidade na árvore para ajustar distância.

Exemplo:

```text id="rtyvke"
root children
distance 1.0

nested children
distance 1.3
```

Somente se melhorar resultado.

---

# Explosion Presets

Adicionar opcionalmente:

```text id="bmxvgd"
25%
50%
100%
```

Além do slider.

Não obrigatório.

---

# HUD

Atualizar Engineering View:

```text id="l3wtga"
MODEL CORE
ONLINE

COMPONENT CORE
ONLINE

EXPLOSION CORE
ONLINE

ASSEMBLY
PARTIAL

EXPLOSION
62%
```

---

# Estados ainda não conectados

Manter:

```text id="m68q6w"
CAD
NÃO CONECTADO

DIGITAL TWIN
NÃO CONECTADO

IOT
NÃO CONECTADO
```

---

# Performance

Evitar alocação excessiva por frame.

Explosion metadata deve ser pré-calculada.

Durante slider/gesto:

apenas atualizar transforms necessários.

Guide lines devem ser eficientes.

---

# Modelos grandes

Se houver muitos componentes:

exploded view pode ficar visualmente caótico.

Não tentar resolver automaticamente toda montagem.

Documentar que qualidade depende da estrutura do GLB.

---

# AI Core

Não integrar nesta versão.

Mas preparar funções internas futuras:

```text id="tx8h0f"
set_explosion_factor
explode_all
explode_component
reassemble
```

Não expor como AI tools ainda.

---

# Segurança

Exploded View é exclusivamente transformação em memória.

Não alterar arquivo original.

Não salvar transforms no GLB.

Não executar código vindo do modelo.

---

# Testes

Adicionar testes para:

1. metadata generation;
2. direction from center;
3. zero-direction fallback;
4. deterministic direction;
5. factor 0;
6. factor 0.5;
7. factor 1;
8. clamp;
9. exact reassembly;
10. no drift;
11. selected subtree;
12. leaf selection;
13. hierarchy depth;
14. guide lines;
15. hidden components;
16. isolation;
17. loss of tracking;
18. model unload;
19. model replacement;
20. animation state.

Criar fixtures simples com múltiplos components.

---

# Teste de reversibilidade

Este teste é obrigatório.

Para cada componente:

```text id="tyckvs"
originalPosition
```

Aplicar:

```text id="67mgy7"
factor = 1
```

Depois:

```text id="u9btkx"
factor = 0
```

Resultado deve coincidir com:

```text id="vaw7p0"
originalPosition
```

dentro de tolerância numérica apropriada.

---

# Documentação

Criar:

```text id="l8euhg"
docs/engineering-core/v0.5.md
```

Documentar:

* objetivo;
* Explosion Core;
* algoritmo;
* metadata;
* hierarchy;
* factor;
* selected explosion;
* guide lines;
* gesture control;
* reversibilidade;
* limitações.

Registrar:

```text id="veut4g"
v0.1 — Hand Interaction — concluída
v0.2 — Spatial Manipulation — concluída
v0.3 — Model Core — concluída
v0.4 — Component Core — concluída
v0.5 — Exploded View
```

Manter:

```text id="u4cl1q"
Known limitation:
calibração gestual ainda requer refinamento.
```

---

# Critérios de aceite

Engineering Core v0.5 só está concluído quando:

1. Explosion Core existir;
2. metadata for calculada;
3. explosão for determinística;
4. Explode All funcionar;
5. slider funcionar;
6. factor 0 representar montagem original;
7. factor 1 representar explosão máxima;
8. valores intermediários funcionarem;
9. reconstrução for exata;
10. drift não ocorrer;
11. animação funcionar;
12. reassemble funcionar;
13. selected explosion funcionar;
14. leaf sem children for tratado;
15. hierarchy for respeitada;
16. seleção continuar funcionando durante explosão;
17. raycasting funcionar em posições explodidas;
18. inspector atualizar;
19. guide lines funcionarem;
20. guide lines puderem ser desligadas;
21. controle por duas mãos funcionar;
22. afastar mãos aumentar factor;
23. aproximar mãos reduzir factor;
24. loss of tracking for seguro;
25. slider funcionar sem webcam;
26. MOVE global continuar funcionando;
27. ROTATE global continuar funcionando;
28. SCALE global continuar funcionando;
29. hide/isolate continuarem funcionando;
30. trocar modelo resetar Explosion State;
31. unload limpar Explosion State;
32. TypeScript não apresentar erros;
33. testes passarem;
34. build funcionar;
35. Tauri iniciar normalmente;
36. demais módulos do Azriel não sofrerem regressão.

---

# Teste final obrigatório

Carregar um GLB com múltiplos componentes.

Confirmar:

```text id="t1mkc4"
EXPLOSION CORE
ONLINE
```

Mover slider:

```text id="k0j7uk"
0 → 25 → 50 → 100
```

Confirmar separação progressiva.

Mover:

```text id="a6ekjz"
100 → 50 → 25 → 0
```

Confirmar reconstrução exata.

Executar:

```text id="aoklrm"
EXPLODIR MONTAGEM
```

Selecionar uma peça enquanto explodida.

Confirmar inspector.

Ativar:

```text id="vpz5hb"
GUIDE LINES
```

Confirmar origem espacial.

Selecionar um grupo com filhos.

Executar:

```text id="ue7l7j"
EXPLODE SELECTED
```

Confirmar explosão apenas da subárvore.

Executar:

```text id="qmhq7x"
RECONSTRUIR
```

Confirmar montagem.

Ativar webcam.

Selecionar:

```text
CONTROL = EXPLODE
```

Mostrar duas mãos.

Afastar as mãos.

Confirmar:

```text
EXPLOSION FACTOR
AUMENTANDO
```

Aproximar as mãos.

Confirmar:

```text
EXPLOSION FACTOR
DIMINUINDO
```

Retirar uma das mãos durante a operação.

Confirmar que:

* a sessão gestual é cancelada;
* nenhuma peça salta;
* o último fator válido é preservado.

Depois testar:

```text
MOVE
ROTATE
SCALE
```

com a montagem parcialmente explodida.

Confirmar que o `ModelRoot` continua manipulável sem perder os offsets dos componentes.

Finalmente executar:

```text
RECONSTRUIR
```

e validar que todos os componentes retornam exatamente às posições originais.

---

# Resultado esperado

Ao concluir a v0.5, o Engineering Core deve conseguir transformar:

```text
MONTAGEM COMPLETA
```

em:

```text
        COMPONENTE A
             ↑

COMPONENTE B ← CORE → COMPONENTE C

             ↓
        COMPONENTE D
```

de forma contínua e reversível.

O operador poderá:

* abrir a montagem;
* controlar quanto ela é aberta;
* selecionar peças enquanto separadas;
* inspecionar componentes;
* reconstruir tudo;
* controlar a explosão com duas mãos.

A evolução passa a ser:

```text
v0.1
MÃO → OBJETO

v0.2
MOVE / ROTATE / SCALE

v0.3
MODELO 3D REAL

v0.4
COMPONENTES

v0.5
MONTAGEM ↔ EXPLODED VIEW
```

A próxima etapa não deve ser adicionada automaticamente.

Primeiro validar a qualidade visual da explosão com diferentes modelos GLB.

Se a estratégia automática produzir resultados bons, a próxima evolução poderá trabalhar com:

# Engineering Core v0.6 — Assembly Intelligence

Possíveis objetivos futuros:

* relações entre componentes;
* grupos mecânicos;
* dependências;
* montagem/desmontagem guiada;
* identificação de componentes;
* integração inicial com o AI Core.

Mas somente após a v0.5 estar estável.

---

# Filosofia da versão

Não queremos apenas afastar meshes aleatoriamente.

Queremos transformar uma montagem complexa em uma representação espacial que seja mais fácil de compreender.

# DESMONTAR PARA ENTENDER.

---

# Estado da implementação — 02/09/2026

Status: **concluída e validada em 02/09/2026**.

Foram entregues o `ExplosionService`, metadata determinística por unidade lógica, explosão `ALL` e `SELECTED`, fator contínuo de 0 a 1, presets, animação, reconstrução exata sem drift, linhas-guia, inspector com offset, seleção e raycasting nas posições explodidas, preservação de hide/isolation e controle por distância relativa entre duas mãos.

O slider permanece como controle principal sem webcam. No modo `EXPLODE`, afastar as duas mãos aumenta o fator e aproximá-las reduz. Se uma mão desaparecer, a sessão é cancelada e o último fator válido permanece aplicado.

MOVE, ROTATE e SCALE do `ModelRoot` continuam funcionando com a montagem aberta. A manipulação gestual individual aprovada na v0.4 continua disponível com fator zero; ao iniciar uma explosão, transforms manuais retornam ao snapshot original e ficam suspensos enquanto o fator for maior que zero, evitando conflito entre transform manual e offset de explosão.

A suíte final possui 107 testes aprovados. Lint, TypeScript, build Vite e inicialização do Tauri foram validados. O aviso já conhecido do chunk do Engineering View acima de 500 kB permanece sem impedir o build.

Known limitation: a calibração gestual ainda requer refinamento e permanece planejada para v0.2.1. A qualidade visual da separação automática depende da estrutura da scene graph do GLB.

Com a aprovação do operador, a Engineering Core v0.5 está encerrada.
