# AZRIEL Finance v0.3 — Planejamento Financeiro

## Prompt de implementação para Codex

### Contexto e objetivo

As versões v0.1 (lançamentos, categorias, classificação PESSOAL/PROFISSIONAL, competência/caixa) e v0.2 (contratos, recorrências, empréstimos, parcelas e compromissos) já foram implementadas. Inspecione o código e a documentação reais antes de alterar qualquer coisa: os nomes e contratos existentes prevalecem sobre exemplos deste documento.

**Objetivo da v0.3:** transformar registros e compromissos em planejamento: orçamentos mensais, metas financeiras, projeções e alertas úteis. Não criar novo subsistema financeiro, nem refazer a v0.1/v0.2.

### 1. Escopo funcional

Implementar, dentro do módulo Finanças existente:

- **Orçamento mensal:** limites por categoria e classificação PESSOAL/PROFISSIONAL, com visão do planejado, realizado, comprometido e disponível no orçamento.
- **Metas financeiras:** objetivos com valor-alvo, prazo opcional e aportes manuais registrados; permitir meta de reserva de emergência sem presumir saldo real.
- **Projeção financeira:** entradas e saídas futuras baseadas em lançamentos previstos, contratos e compromissos já cadastrados; projeção de resultado acumulado a partir de um ponto inicial explícito, não de saldo bancário inventado.
- **Cenários:** simular mudanças em receita, custo ou compromisso sem modificar dados reais.
- **Alertas internos:** vencimentos próximos, compromissos atrasados, orçamento próximo do limite/ultrapassado e metas fora do ritmo previsto quando houver dados suficientes.
- **Relatórios:** comparativo mensal planejado x realizado, por categoria e PESSOAL/PROFISSIONAL, além de previsão para meses seguintes.

Não criar contas bancárias, saldos de bancos, integração bancária, Open Finance, automação de investimentos, IA ou contabilidade fiscal completa.

### 2. Inspeção obrigatória

Antes de codificar, identificar: tabelas/migrations e DTOs da v0.1/v0.2, regras de competência e caixa, status de lançamentos, geração de recorrências, contratos, empréstimos, filtros, frontend, stores, commands Rust/Tauri, padrões de validação e testes, design system e política de datas/timezone. Apresentar plano curto e seguir a arquitetura existente. Não criar abstrações paralelas.

### 3. Orçamentos mensais

Entidade conceitual `FinancialBudget`:

- `id`, `year_month` (YYYY-MM), `scope` (PERSONAL/PROFESSIONAL), `category_id`, `limit_cents`, `created_at`, `updated_at`.

Permitir criar, editar, excluir com confirmação e copiar limites de um mês anterior para o seguinte mediante ação explícita. Impedir duplicidade para a combinação mês + classificação + categoria. Usar categorias reais do sistema; se categoria for desativada, preservar orçamento/histórico.

A visão principal deve distinguir:

- **Limite planejado:** valor configurado para o mês.
- **Realizado:** despesas efetivamente pagas no mês, conforme a visão de caixa; quando o usuário escolher competência, indicar claramente que o recorte mudou.
- **Comprometido:** despesas ainda não pagas, mas previstas/vencendo no período, sem incluir novamente valores já pagos.
- **Disponível:** limite menos realizado menos comprometido, quando esses conceitos forem exibidos juntos na mesma base temporal.

Exibir excedentes negativos de forma clara. Orçamento não é saldo de conta e não cria despesa automaticamente.

### 4. Metas financeiras

Entidade conceitual `FinancialGoal`:

- `id`, `title`, `scope`, `target_cents`, `target_date` opcional, `status` (ACTIVE/COMPLETED/ARCHIVED), `created_at`, `updated_at`.

Aportes manuais em entidade/registro próprio com `goal_id`, `amount_cents`, `date`, `note` opcional. Os aportes são **marcadores de progresso**, não lançamentos de saída/transferência automática. Deixar explícito na UI que uma meta preenchida manualmente não comprova dinheiro disponível em banco. Não somar aportes ao fluxo de caixa. Permitir ajustar/corrigir aportes com histórico coerente. Meta concluída não deve alterar o patrimônio automaticamente.

Exibir valor-alvo, progresso registrado, falta para atingir e prazo, quando houver. Sem taxas de retorno presumidas.

### 5. Projeção de entradas e saídas

A projeção deve consumir os dados reais da v0.1/v0.2, sem duplicá-los:

- lançamentos previstos ainda não realizados;
- parcelas e compromissos futuros;
- ocorrências de contratos e recorrências, somente quando ainda não representadas por lançamento/ocorrência persistida;
- pagamentos realizados como histórico, nunca novamente como previsão futura.

Definir identificador estável da origem de cada item projetado (tipo + ID + ocorrência/período), reutilizando a deduplicação existente. Projeção é cálculo/visualização; **não materializar novos lançamentos apenas para desenhar um gráfico**.

Oferecer horizonte simples de 3, 6 e 12 meses. Separar projeção por competência e por caixa, identificando qual base está ativa. Para eventos com vencimento conhecido e pagamento ainda desconhecido, mostrar previsão pela data esperada e deixar claro que não é confirmação de recebimento/pagamento.

Exibir:

- receitas projetadas;
- despesas projetadas;
- resultado projetado do período;
- resultado acumulado projetado a partir de zero **ou** de valor-base informado manualmente e rotulado como simulação.

Nunca chamar resultado acumulado de **saldo bancário**, **dinheiro em conta** ou **patrimônio**. Não inferir que a renda de referência R$ 6.000 foi recebida ou contratada.

### 6. Cenários

Criar simulador leve e temporário, com cenários como:

- receita profissional reduzida em X%;
- custos operacionais aumentados em X%;
- novo compromisso mensal de R$ Y;
- encerramento hipotético de contrato específico, se já houver vínculo na v0.2.

Cenários usam dados-base mais ajustes explícitos e apresentam diferença frente ao cenário-base. Não alteram contratos, recorrências, orçamentos nem lançamentos reais. Persistência de cenários é opcional apenas se houver padrão existente e benefício claro; não construir motor complexo de forecasting.

### 7. Alertas internos

Mostrar no próprio módulo:

- vencimento próximo (janela configurável simples ou padrão documentado);
- vencido e não pago;
- orçamento >= 80% do limite e > 100%, sem alertas duplicados desnecessários;
- meta com prazo e ritmo insuficiente, apenas quando houver aporte/tempo suficientes para cálculo compreensível.

Alertas são derivados dos dados existentes e recalculados ao abrir/atualizar; não criar notificações externas, automações em background ou mensagens de IA nesta versão. Status de alerta não deve alterar lançamentos.

### 8. UI/UX

Manter o módulo Finanças atual. Adicionar áreas internas discretas, conforme navegação existente:

- **Planejamento**: orçamento e metas;
- **Projeções**: entradas/saídas e cenários;
- **Alertas**: resumo contextual no dashboard e lista curta.

Evitar excesso de cards e indicadores. Mostrar valores reais/derivados, estados vazios claros e separação PESSOAL/PROFISSIONAL. Valores futuros devem ser identificados como **previstos**, não realizados. Não duplicar controles de contratos ou lançamentos que já existem na v0.2.

### 9. Regras técnicas

- Valores monetários em **centavos inteiros**, sem float para armazenar ou agregar dinheiro.
- Datas e timezone consistentes com v0.1/v0.2; testes para virada de mês/ano.
- Cancelados não entram em totais; pagos não reaparecem como pendências.
- Competência e caixa não devem ser somados entre si.
- Recorrências e contratos não podem gerar dupla contagem.
- SQLite: migrations incrementais, constraints, FKs e índices adequados às consultas reais.
- Reutilizar commands/services/repositories/componentes do projeto.
- Erros de persistência devem aparecer na UI; não simular sucesso.
- Nenhuma integração externa ou envio de dados financeiros à IA.

### 10. Testes obrigatórios

Validar, com testes coerentes com o projeto:

1. orçamento criado/editado e unicidade por mês+escopo+categoria;
2. orçamento sem lançamentos e com categoria desativada;
3. realizado versus previsto/comprometido sem dupla contagem;
4. competência versus caixa e virada de mês/ano;
5. meta, aportes, correções, arquivamento e ausência de movimentação bancária implícita;
6. contratos/recorrências/parcelas representados uma única vez na projeção;
7. pagamento confirmado remove item correspondente da previsão pendente;
8. horizonte de 3/6/12 meses;
9. cenário não altera dados reais;
10. alertas de vencimento, atraso e orçamento;
11. valores zerados, negativos derivados, arredondamento e grandes valores;
12. persistência após reinício, migrations com banco existente e banco novo;
13. regressão das telas e cálculos v0.1/v0.2;
14. build e inicialização Tauri.

### 11. Critérios de aceite

A v0.3 está concluída quando:

- orçamento mensal por categoria e classificação funciona;
- comparativo planejado/realizado/comprometido é coerente;
- metas e aportes manuais funcionam sem fingir saldo real;
- projeção futura utiliza dados existentes sem duplicação;
- competência e caixa são explicitamente diferenciados;
- cenários são simulações isoladas;
- alertas internos são úteis e corretos;
- nenhum banco/saldo/conta é inventado;
- v0.1 e v0.2 continuam funcionais;
- migrations, testes e build passam;
- documentação e limitações são registradas.

### 12. Documentação e entrega

Criar `docs/finance/v0.3.md` ou equivalente ao padrão do repositório, incluindo fórmulas, origens de projeção, deduplicação, semântica de datas, limitações e testes.

Ao concluir, o Codex deve resumir: arquivos criados/alterados, migrations, telas, cálculos, decisões de integração, testes, build/Tauri e limitações. Não avançar para cadastro de contas bancárias nem v0.4 automaticamente.

**Resultado esperado:** planejamento financeiro pessoal e profissional confiável, que ajude a decidir quanto gastar, quanto reservar e quais compromissos futuros podem pressionar o orçamento — sem confundir projeções com dinheiro efetivamente disponível.

---

## Estado da implementação — 14/10/2026

Status: **implementada; aguardando validação operacional do usuário**.

A implementação utiliza a migration `0044_finance_planning.sql`, mantém lançamentos e compromissos anteriores como fontes de verdade, calcula ocorrências futuras sem persistir dados apenas para visualização e conserva cenários temporários fora do banco. A documentação técnica está em `Docs/financeiro/v0.3.md`.
