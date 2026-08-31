import type { KnowledgeArea } from "../types";

export const knowledgeAreas: KnowledgeArea[] = [
  { id: "cyber", name: "Cibersegurança", group: "Computação", coverage: 90, depth: 80, priority: "low", projectIds: [] },
  { id: "programming", name: "Programação", group: "Computação", coverage: 80, depth: 65, priority: "low", projectIds: ["azriel", "atlas3d"] },
  { id: "ai", name: "IA / Software", group: "Computação", coverage: 60, depth: 40, priority: "medium", projectIds: ["azriel"] },
  { id: "biology", name: "Biologia / Genética", group: "Biociências", coverage: 45, depth: 30, priority: "medium", projectIds: ["mendel-lab", "gene-expression", "genescope"] },
  { id: "biomed", name: "Biomedicina / Biotecnologia", group: "Biociências", coverage: 35, depth: 20, priority: "medium", projectIds: ["pcr-simulator", "gene-expression"] },
  { id: "physics", name: "Física Aplicada", group: "Fundamentos", coverage: 30, depth: 20, priority: "high", projectIds: ["arccore"] },
  { id: "energy", name: "Energia", group: "Engenharia", coverage: 25, depth: 15, priority: "medium", projectIds: ["arccore"] },
  { id: "robotics", name: "Robótica", group: "Engenharia", coverage: 20, depth: 10, priority: "medium", projectIds: [] },
  { id: "electronics", name: "Eletrônica", group: "Engenharia", coverage: 20, depth: 10, priority: "high", projectIds: ["arccore"] },
  { id: "control", name: "Controle / Automação", group: "Engenharia", coverage: 20, depth: 10, priority: "medium", projectIds: ["azriel", "arccore"] },
  { id: "materials", name: "Materiais", group: "Engenharia", coverage: 15, depth: 8, priority: "medium", projectIds: ["arccore"] },
  { id: "electrical", name: "Engenharia Elétrica", group: "Engenharia", coverage: 15, depth: 5, priority: "critical", projectIds: ["arccore"] },
  { id: "mechanical", name: "Engenharia Mecânica", group: "Engenharia", coverage: 15, depth: 5, priority: "critical", projectIds: [] },
];

export const gapAreas = knowledgeAreas.filter((area) => ["critical", "high"].includes(area.priority));
