# AZRIEL — Engineering Core v0.1

A interface visual do **Engineering View** já existe e está aprovada.

Agora implemente a primeira prova funcional do módulo.

# Objetivo

Provar que a webcam do computador consegue detectar uma mão e usar um gesto de pinça para manipular um objeto 3D dentro do Engineering View.

O critério principal desta versão é simples:

> **A mão deve conseguir pegar e mover um objeto digital na tela.**

Não aumentar o escopo além disso.

---

# Antes de alterar código

1. leia o README;
2. examine a arquitetura atual do Azriel;
3. localize o módulo Engineering View;
4. preserve completamente o visual existente;
5. não redesenhe a tela;
6. entenda a stack React + TypeScript + Tauri;
7. rode lint, testes e build antes de iniciar;
8. preserve todos os módulos funcionais existentes.

---

# Escopo

Implementar:

* acesso à webcam;
* detecção de uma mão;
* landmarks da mão;
* cursor virtual;
* gesto de pinça;
* gesto de release;
* cena 3D simples;
* objeto 3D simples;
* seleção do objeto pela pinça;
* movimentação do objeto;
* suavização do tracking;
* tratamento de perda de tracking.

Não implementar ainda:

* duas mãos;
* rotação por gesto;
* escala por gesto;
* GLB/GLTF;
* CAD real;
* exploded view;
* IoT;
* Digital Twin;
* integração com AI Core;
* voz;
* controle do mouse do Windows.

---

# Stack

Para hand tracking:

* MediaPipe Hand Landmarker, preferencialmente a implementação oficial para web;
* TypeScript.

Para 3D:

* Three.js;
* React Three Fiber se fizer sentido na arquitetura atual.

Não adicionar dependências pesadas sem necessidade.

---

# Arquitetura

Separar responsabilidades.

Estrutura conceitual:

```text
Engineering View
        │
        ├── Camera Service
        ├── Hand Tracking
        ├── Gesture Engine
        ├── Interaction Controller
        └── 3D Scene
```

Não colocar toda a lógica em um único componente.

---

# Camera Service

Criar camada responsável por:

* solicitar permissão;
* iniciar câmera;
* parar câmera;
* detectar câmera indisponível;
* tratar permissão negada;
* tratar câmera ocupada;
* expor stream para o hand tracking.

A webcam deve ser utilizada apenas localmente.

Nenhum frame pode ser enviado para:

* Ollama;
* APIs;
* servidores;
* AI Core;
* serviços externos.

---

# Estado visual da câmera

Integrar com o HUD atual.

Quando inativa:

```text
CAMERA
OFFLINE
```

Quando solicitando permissão:

```text
CAMERA
REQUESTING
```

Quando ativa:

```text
CAMERA
ONLINE

PROCESSAMENTO
LOCAL
```

---

# Preview

Adicionar um preview pequeno da webcam em área secundária da interface.

Não cobrir o viewport principal.

Permitir:

```text
CAMERA PREVIEW
ON / OFF
```

O preview deve ser espelhado horizontalmente para tornar a interação natural.

---

# Hand Tracking

Detectar inicialmente apenas uma mão.

Obter os 21 landmarks.

Criar tipos claros.

Exemplo conceitual:

```ts
interface HandLandmark {
  x: number;
  y: number;
  z: number;
}

interface TrackedHand {
  landmarks: HandLandmark[];
  handedness?: "left" | "right";
  confidence?: number;
}
```

Evitar `any`.

---

# Debug de landmarks

Adicionar modo:

```text
HAND DEBUG
ON / OFF
```

Quando ativo:

mostrar no preview:

* landmarks;
* conexões entre os dedos;
* handedness;
* confiança;
* FPS aproximado.

Quando desligado:

tracking continua funcionando normalmente.

---

# Cursor virtual

Criar um cursor controlado pela mão.

Utilizar inicialmente a posição da ponta do dedo indicador.

O cursor deve existir somente dentro do Engineering View.

Não controlar o cursor real do Windows.

Visual:

* pequeno círculo técnico;
* cruz;
* retículo HUD.

Exemplo:

```text
+
```

---

# Conversão de coordenadas

MediaPipe retorna coordenadas normalizadas.

Criar transformação explícita:

```text
MediaPipe coordinates
        ↓
screen / viewport coordinates
        ↓
3D interaction coordinates
```

Levar em conta o espelhamento da câmera.

Não espalhar transformações matemáticas pelo código.

Centralizar em utility/service apropriado.

---

# Gesture Engine

Criar camada separada.

Nesta versão implementar apenas:

```text
PINCH
RELEASE
```

---

# PINCH

Usar a distância entre:

* ponta do polegar;
* ponta do indicador.

Quando a distância ficar abaixo de um threshold:

```text
PINCH = ACTIVE
```

---

# RELEASE

Quando a distância ultrapassar outro threshold:

```text
PINCH = INACTIVE
```

Usar thresholds diferentes para entrada e saída.

Isso deve criar hysteresis e evitar flickering.

---

# Exemplo

Não usar apenas:

```text
distance < 0.05
```

para alternar continuamente.

Usar algo conceitualmente semelhante:

```text
PINCH_START_THRESHOLD
PINCH_RELEASE_THRESHOLD
```

com:

```text
releaseThreshold > startThreshold
```

---

# Configuração

Centralizar:

* pinch threshold;
* release threshold;
* smoothing;
* confidence mínima.

Não colocar valores mágicos espalhados.

---

# HUD do tracking

Atualizar lateral/estado do Engineering View para mostrar dados reais.

Exemplo:

```text
HAND TRACKING

HAND           DETECTED
SIDE           RIGHT
GESTURE        PINCH
CONFIDENCE     0.94
FPS            28
```

Quando não houver mão:

```text
HAND
NOT DETECTED

GESTURE
NONE
```

---

# Cena 3D

Substituir o estado visual:

```text
SEM MODELO CARREGADO
```

por uma cena técnica real quando o renderer estiver ativo.

Criar inicialmente:

* grid;
* câmera perspectiva;
* iluminação simples;
* um objeto de teste.

Preservar a estética atual do viewport.

---

# Objeto de teste

Criar um objeto chamado:

```text
TEST-01
```

Pode ser:

* cubo;
* geometria técnica simples.

Mostrar informações:

```text
OBJECT
TEST-01

STATUS
READY

CONTROL
HAND
```

---

# Interação

Fluxo obrigatório:

```text
mão detectada
↓
cursor virtual ativo
↓
cursor passa sobre TEST-01
↓
objeto entra em HOVER
↓
usuário faz PINCH
↓
objeto entra em GRABBED
↓
movimento da mão move o objeto
↓
usuário abre os dedos
↓
RELEASE
↓
objeto permanece na última posição
```

---

# Hover

Antes de permitir grab:

o cursor deve estar sobre/intersectar o objeto.

Mostrar estado visual discreto:

```text
OBJECT STATUS
TARGETED
```

---

# Grab

Quando pinch ocorrer sobre o objeto:

```text
OBJECT STATUS
GRABBED
```

Enquanto `PINCH` permanecer ativo:

o objeto acompanha a mão.

---

# Release

Quando a pinça for liberada:

```text
OBJECT STATUS
READY
```

O objeto deve permanecer na posição atual.

Não retornar automaticamente ao centro.

---

# Reset

Adicionar botão:

```text
RESET OBJECT
```

que restaura:

* posição;
* orientação;
* estado.

---

# Smoothing

Tracking de webcam possui jitter.

Implementar smoothing simples.

Preferir:

* exponential smoothing;
* lerp.

Não implementar filtro complexo nesta versão.

O objetivo é tornar o movimento visualmente estável sem criar atraso excessivo.

---

# Loss of Tracking

Se a mão desaparecer durante um grab:

NÃO continuar movendo o objeto.

Comportamento:

```text
HAND LOST
↓
grab cancelado
↓
objeto permanece na última posição conhecida
```

---

# Estados

Criar estados claros.

## Engineering Core

```text
offline
requesting_camera
tracking
ready
error
```

## Hand

```text
not_detected
detected
```

## Gesture

```text
none
pinch
```

## Object

```text
ready
targeted
grabbed
```

---

# Botões

Adicionar/ativar:

```text
INICIAR CÂMERA
PARAR CÂMERA
CAMERA PREVIEW ON/OFF
HAND DEBUG ON/OFF
RESET OBJECT
```

Todos devem funcionar.

---

# Performance

Não rodar processamento desnecessariamente.

O módulo deve poder permanecer aberto por períodos longos.

Evitar:

* loops duplicados;
* requestAnimationFrame duplicado;
* listeners sem cleanup;
* processamento quando câmera estiver parada.

---

# Segurança e privacidade

Engineering Core NÃO pode:

* controlar mouse do sistema;
* controlar teclado;
* acessar arquivos arbitrários;
* executar automações;
* enviar frames ao Ollama;
* enviar frames para internet.

A câmera é apenas entrada local de visão computacional.

---

# Integração com AI Core

Não integrar nesta versão.

Mas preparar interfaces internas futuras para:

```text
select_object
move_object
rotate_object
scale_object
reset_object
explode_model
```

Não expor como tools ainda.

---

# Registro visual

Atualizar a seção direita já existente.

Hoje existem estados como:

```text
CAD
NÃO CONECTADO

DIGITAL TWINS
NÃO CONECTADO

IOT
NÃO CONECTADO
```

Manter esses estados.

Adicionar/atualizar apenas:

```text
RENDERIZADOR 3D
ONLINE

HAND TRACKING
ONLINE
```

quando os respectivos sistemas estiverem ativos.

---

# Compatibilidade

Prioridade:

Windows + Tauri atual.

Usar a webcam disponível no sistema.

Não introduzir dependência de ARCore.

Essa versão não é AR.

É:

```text
COMPUTER VISION
+
HAND TRACKING
+
3D INTERACTION
```

---

# Testes

Adicionar testes unitários para lógica isolável.

Prioridades:

1. distância entre landmarks;
2. pinch start;
3. pinch release;
4. hysteresis;
5. smoothing;
6. transformação de coordenadas;
7. mirror;
8. loss of tracking;
9. mudança de estado ready → targeted;
10. mudança targeted → grabbed;
11. release;
12. reset.

Não exigir webcam real nos testes.

Utilizar landmarks simulados.

---

# Não implementar

Fora do escopo:

* duas mãos;
* escala;
* rotação por mão;
* GLB;
* GLTF;
* modelos CAD;
* seleção de componentes internos;
* exploded view;
* física;
* Digital Twin;
* telemetria;
* IoT;
* comandos de voz;
* AI Core;
* reconhecimento de gestos complexos.

---

# Documentação

Criar:

```text
docs/engineering-core/v0.1.md
```

Documentar:

* objetivo;
* arquitetura;
* MediaPipe;
* Three.js;
* Gesture Engine;
* thresholds;
* hysteresis;
* smoothing;
* transformação de coordenadas;
* privacidade;
* limitações;
* como testar.

---

# Critérios de aceite

Engineering Core v0.1 só está concluído quando:

1. Engineering View existente for preservado;
2. câmera puder ser iniciada;
3. câmera puder ser parada;
4. permissão negada for tratada;
5. mão for detectada;
6. landmarks forem obtidos;
7. debug funcionar;
8. cursor virtual seguir o indicador;
9. pinch for detectado;
10. release for detectado;
11. hysteresis evitar flickering;
12. renderer 3D estiver ativo;
13. TEST-01 existir;
14. hover for detectado;
15. pinch sobre objeto gerar grab;
16. objeto acompanhar movimento da mão;
17. release liberar objeto;
18. objeto permanecer na posição;
19. smoothing reduzir jitter;
20. loss of tracking cancelar grab;
21. reset funcionar;
22. preview puder ser ativado/desativado;
23. frames permanecerem locais;
24. AI Core não receber câmera;
25. TypeScript não apresentar erros;
26. testes passarem;
27. build funcionar;
28. Tauri iniciar normalmente;
29. demais módulos do Azriel continuarem funcionando.

---

# Teste final obrigatório

Abrir:

```text
Engineering View
```

Executar:

```text
INICIAR CÂMERA
```

Confirmar:

```text
CAMERA ONLINE
HAND TRACKING ONLINE
```

Colocar uma mão diante da webcam.

Confirmar:

```text
HAND DETECTED
```

Mover o indicador.

Confirmar que o cursor acompanha.

Passar o cursor sobre:

```text
TEST-01
```

Confirmar:

```text
TARGETED
```

Fazer pinça com polegar + indicador.

Confirmar:

```text
PINCH
GRABBED
```

Mover a mão.

Confirmar que TEST-01 acompanha.

Soltar os dedos.

Confirmar:

```text
RELEASE
READY
```

O objeto deve permanecer onde foi solto.

Retirar a mão da câmera durante outro grab.

Confirmar que o objeto para e não salta.

Executar:

```text
RESET OBJECT
```

Confirmar retorno à posição inicial.

Desativar debug.

Confirmar que a interação continua funcionando.

Parar câmera.

Confirmar que o módulo retorna ao estado seguro sem erros.

---

# Resultado esperado

Ao concluir esta versão, o Engineering Core deve provar uma única capacidade fundamental:

# A MÃO DO OPERADOR CONSEGUE PEGAR E MOVER UM OBJETO DIGITAL.

Não precisamos ainda de um motor.

Não precisamos ainda de CAD.

Não precisamos ainda de holograma.

Primeiro provamos a interação.

Depois evoluímos.
