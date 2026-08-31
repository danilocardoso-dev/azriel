import type { ResearchItem } from "../types";

export const researchItems: ResearchItem[] = [
  { id: "BIO-P01", title: "Mendel Lab", domain: "Genética + Software", objective: "Simular cruzamentos e probabilidades", status: "project", impact: "Genética educacional" },
  { id: "BIO-P02", title: "Gene Expression Explorer", domain: "Bioinformática", objective: "Comparar expressão gênica", status: "project", impact: "Transcriptômica + dados" },
  { id: "BIO-P03", title: "PCR Simulator", domain: "Biologia Molecular", objective: "Modelar primers, ciclos e amplicons", status: "project", impact: "Molecular + software" },
  { id: "BIO-P04", title: "GeneScope", domain: "Bioinformática", objective: "Explorar variantes genéticas", status: "project", impact: "Genética aplicada" },
  { id: "FND-01", title: "Matemática e Física", domain: "Fundamentos", objective: "Preparar a base para Mecatrônica", status: "study", impact: "Redução de lacunas" },
  { id: "ENE-01", title: "ArcCore", domain: "Energia", objective: "Investigar armazenamento e gestão", status: "queue", impact: "Projeto integrador" },
];
