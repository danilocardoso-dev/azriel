# AZRIEL Finance v0.2 — Contratos e Compromissos

> Especificação de implementação para Codex. Evolução incremental do Finance v0.1, sem criar subsistema paralelo.

## 1. Contexto e objetivo

A v0.1 já oferece lançamentos de receitas/despesas, classificação `PERSONAL`/`PROFESSIONAL`, categorias, competência, vencimento/pagamento, previsto/realizado, dashboard e relatórios. **Inspecione o repositório antes de codificar**: confirme nomes, estados, banco, migrations, serviços, comandos Tauri, frontend e padrões de UI. O código existente é a fonte de verdade.

A v0.2 deve responder:

- Quais contratos geram receita, quanto e até quando?
- O que tenho a receber nos próximos meses?
- Quais custos recorrentes mantenho, especialmente ferramentas, servidores e MEI?
- Quais parcelas e dívidas ainda preciso pagar?
- Quais compromissos estão próximos ou atrasados?

**Não implementar** contas bancárias, saldo bancário, integração Open Finance, sincronização bancária, contabilidade completa, emissão de notas, IA ou planejamento avançado.

## 2. Princípios inegociáveis

1. Preservar todos os lançamentos e relatórios v0.1.
2. Valores em **centavos inteiros**; não usar float para dinheiro.
3. Separar **obrigação prevista** de **pagamento/recebimento efetivo**.
4. Reutilizar o mecanismo de `FinancialTransaction` da v0.1 para movimentos efetivos. Não criar um segundo livro-caixa.
5. Gerar compromissos sem duplicatas, mesmo após reiniciar ou repetir uma operação.
6. Uma recorrência ou contrato **não é dinheiro recebido**. Só uma baixa confirmada produz lançamento realizado.
7. Preservar distinção competência versus caixa e PESSOAL versus PROFISSIONAL.
8. Evitar novas telas excessivamente densas. Seguir design system AZRIEL.

## 3. Escopo funcional

### 3.1 Contratos de receita

Cadastro com:

- `id`, nome/identificador, contraparte (nome livre), descrição opcional;
- classificação padrão `PROFESSIONAL` (permitir `PERSONAL` quando necessário);
- valor por cobrança em centavos, moeda BRL;
- periodicidade inicial `MONTHLY`, `WEEKLY`, `YEARLY` ou `ONE_TIME`, apenas se viável no padrão existente;
- início, fim opcional, dia/data de vencimento, status `ACTIVE`, `PAUSED`, `ENDED`, `CANCELLED`;
- categoria de receita e observações opcionais;
- timestamps.

Não exigir cadastro de cliente/CRM. A contraparte é apenas um campo descritivo. Contrato com fim definido não gera vencimentos depois da vigência. Contrato pausado não gera novos compromissos no período pausado, sem apagar os antigos.

Exibir total **contratado previsto**, **a receber**, **recebido** e **atrasado**, sempre identificando o período. Não somar todo o valor futuro de contrato sem prazo como se fosse saldo disponível.

### 3.2 Despesas recorrentes

Cadastro de custos como servidor, hospedagem, domínio, ferramenta, assinatura, tributo MEI e outras despesas.

Campos: `id`, título, categoria, `scope`, valor em centavos, periodicidade, início, fim opcional, próximo vencimento/regra de vencimento, status e observações. Permitir marcar valores variáveis como **estimados** e corrigir o valor de uma ocorrência sem alterar automaticamente toda a série.

### 3.3 Empréstimos e parcelamentos

Cadastro mínimo:

- identificação, credor, finalidade, classificação pessoal/profissional;
- principal original, saldo devedor inicial na data de cadastro, data de referência;
- número de parcelas, vencimentos, valor das parcelas, taxa de juros **somente se conhecida**, status;
- observações.

**Não calcular cronograma de amortização presumindo SAC/Price ou juros compostos sem dados suficientes.** Preferir cadastro do cronograma real de parcelas; quando não disponível, permitir parcelas informadas manualmente e marcar saldo como **informado/estimado**, nunca como saldo bancário confirmado. Diferenciar principal, juros e encargos quando fornecidos. Evitar somar valor de empréstimo recebido à renda mensal sem classificação explícita como movimentação financeira não operacional.

Baixa de parcela deve produzir/associar uma única despesa da v0.1. Pagamento parcial: implementar apenas se a arquitetura permitir com consistência; caso contrário documentar como limitação e não simular quitação integral.

### 3.4 Compromissos

Criar visão unificada de ocorrências:

- receitas previstas de contratos;
- despesas recorrentes;
- parcelas de empréstimos.

Cada ocorrência precisa de `id`, origem (`CONTRACT`, `RECURRING_EXPENSE`, `LOAN_INSTALLMENT`), ID da origem, chave única de período/parcela, valor previsto, competência, vencimento, status (`PENDING`, `SETTLED`, `OVERDUE` derivado, `CANCELLED`), `transaction_id` opcional e timestamps.

`OVERDUE` deve ser derivado de vencimento e situação, não sobrescrever um status persistido arbitrariamente. A origem e a chave de ocorrência devem possuir restrição de unicidade no SQLite.

## 4. Geração de ocorrências

- Gerar em janela limitada (ex.: mês atual + próximos 3 meses), sob demanda ou por operação controlada; não gerar infinitamente.
- Reexecutar geração deve ser idempotente.
- Tratar corretamente meses sem dia 29/30/31, anos bissextos, fuso local e datas de competência. Definir e documentar regra de ajuste para último dia do mês.
- Editar regra futura não deve reescrever pagamentos já confirmados.
- Pausar/cancelar origem não deve apagar histórico quitado.
- Para ocorrências futuras ainda não liquidadas, explicitar a política de atualização quando a regra/valor mudar.

## 5. Baixa e integração com v0.1

Fluxo recomendado:

1. Usuário abre compromisso.
2. Confirma valor efetivo e data de pagamento/recebimento.
3. Backend cria **uma** `FinancialTransaction` com tipo, categoria, scope, competência e `paid_at` coerentes.
4. Backend associa a ocorrência à transação e marca quitada.
5. Tudo ocorre na **mesma transação SQLite**.

Evitar duplicidade por duplo clique, retry ou reinício. Se o lançamento v0.1 for editado/cancelado, definir política explícita para sincronizar ou reabrir o compromisso, sem deixar indicadores inconsistentes.

Permitir associar uma transação v0.1 já existente a uma ocorrência, com verificação de compatibilidade e prevenção de vínculo múltiplo indevido. Não obrigar o usuário a lançar a mesma despesa duas vezes.

## 6. Dashboard e relatórios

Adicionar, sem substituir a visão v0.1:

- a receber nos próximos 30 dias;
- a pagar nos próximos 30 dias;
- vencidos a receber e a pagar;
- contratos ativos e receita contratada mensal equivalente **quando calculável**;
- custos recorrentes profissionais mensais equivalentes **quando calculável**;
- parcelas futuras e saldo devedor **informado**.

Separar **previsto**, **realizado**, **competência** e **caixa**. Não contar o mesmo pagamento como receita/despesa realizada duas vezes (uma no compromisso e outra no lançamento). Indicadores devem ser derivados das ocorrências e/ou transações com regras documentadas.

## 7. Interface

Dentro de **Finanças**, acrescentar acesso a:

- **Contratos**: lista, detalhe, criar/editar, pausar/encerrar, ocorrências relacionadas;
- **Recorrências**: custos fixos e assinaturas, status, próximos vencimentos;
- **Empréstimos**: dívida informada, parcelas, próximas datas e baixas;
- **Compromissos**: lista cronológica, filtros por período, tipo, scope e status, ação de confirmar pagamento/recebimento.

Evitar transformar a página inicial em uma ficha contábil. Mostrar indicadores essenciais e permitir detalhamento sob demanda. Estados vazios e mensagens de erro devem ser claros.

## 8. Persistência e arquitetura

Seguir os padrões existentes de Rust/Tauri/SQLite e frontend. Migrations incrementais preservando dados v0.1. Modelos conceituais: `FinanceContract`, `FinanceRecurrence`, `FinanceLoan`, `FinanceLoanInstallment` (se necessário) e `FinanceCommitment`. Adaptar nomes e separação de tabelas ao repositório; não criar abstrações genéricas sem uso.

Adicionar índices por vencimento/status/origem conforme consultas reais. Foreign keys e restrições de unicidade. Sem chamadas de rede ou APIs externas. Sem credenciais financeiras.

## 9. Validação e testes

Cobrir:

- contrato mensal com início/fim, pausa, encerramento e alteração de valor;
- recorrência em dia 31 e fevereiro/ano bissexto;
- geração repetida sem duplicar;
- empréstimo com cronograma informado e parcelas quitadas;
- empréstimo sem taxa conhecida não inventa juros;
- baixa cria exatamente uma transação v0.1;
- baixa repetida não cria duplicata;
- associação a lançamento existente;
- edição/cancelamento de lançamento vinculado;
- vencidos calculados corretamente;
- classificação PESSOAL/PROFISSIONAL;
- competência diferente da data efetiva de caixa;
- totais e filtros sem dupla contagem;
- migração de banco existente e inicialização de banco novo;
- regressão de dashboard, relatórios e CRUD v0.1;
- build e inicialização Tauri.

## 10. Critérios de aceite

A v0.2 está concluída quando:

1. Contratos ativos podem ser cadastrados e consultados.
2. Receitas previstas por contrato aparecem em compromissos.
3. Despesas recorrentes geram vencimentos sem duplicação.
4. Empréstimos e parcelas podem ser acompanhados sem juros inventados.
5. Pagamento/recebimento confirmado cria ou associa uma única transação v0.1.
6. Dashboard distingue previsão de realizado e caixa de competência.
7. Pessoal e profissional continuam separados.
8. Histórico permanece após pausa, edição ou encerramento.
9. Não existem contas bancárias, IA ou integrações bancárias novas.
10. Testes, migrations, build e Tauri passam.
11. Documentação técnica registra regras de recorrência, baixa, sincronização e limitações.

## 11. Documentação e entrega Codex

Criar `docs/finance/v0.2.md` (ou caminho equivalente) com entidades, regras, migrations, semântica financeira, UI, testes e limitações. Ao finalizar, resumir arquivos alterados, migrations, comandos, telas, testes e resultado do build.

**Próximas versões — não implementar:** v0.3 planejamento, orçamento e metas; futuramente contas bancárias, saldos e conciliação.

**Resultado esperado:** controlar contratos, custos recorrentes e dívidas, com visão confiável dos próximos compromissos, sem perder a simplicidade do AZRIEL Finance.

---

## Estado da implementação — 13/10/2026

Status: **implementada; aguardando validação operacional do usuário**.

A implementação utiliza a migration `0043_finance_commitments.sql`, preserva `FinancialTransaction` como livro-caixa único, gera ocorrências idempotentes em janela limitada e integra baixa/vínculo/cancelamento em transações SQLite. A documentação técnica está em `Docs/financeiro/v0.2.md`.
