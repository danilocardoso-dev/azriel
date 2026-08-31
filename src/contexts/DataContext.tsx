import { useCallback, useEffect, useMemo, useState, type ReactNode } from "react";
import { educationService } from "../services/educationService";
import { getDatabaseInfo } from "../services/databaseService";
import { knowledgeService } from "../services/knowledgeService";
import { projectService } from "../services/projectService";
import type { DatabaseInfo, EducationItem, KnowledgeArea, Project } from "../types";
import { DataContext, type DataContextValue } from "./data-context";
import { notifyDataRelationsChanged } from "./dataEvents";
const messageOf = (error: unknown) => error instanceof Error ? error.message : String(error);

export function DataProvider({ children }: { children: ReactNode }) {
  const [projects, setProjects] = useState<Project[]>([]);
  const [knowledgeAreas, setKnowledgeAreas] = useState<KnowledgeArea[]>([]);
  const [education, setEducation] = useState<EducationItem[]>([]);
  const [databaseInfo, setDatabaseInfo] = useState<DatabaseInfo | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const reload = useCallback(async () => {
    setLoading(true); setError(null);
    try {
      const [nextProjects, nextKnowledge, nextEducation, info] = await Promise.all([
        projectService.list(), knowledgeService.list(), educationService.list(), getDatabaseInfo(),
      ]);
      setProjects(nextProjects); setKnowledgeAreas(nextKnowledge); setEducation(nextEducation); setDatabaseInfo(info);
    } catch (reason) { setError(messageOf(reason)); }
    finally { setLoading(false); }
  }, []);

  useEffect(() => {
    const initialization = window.setTimeout(() => void reload(), 0);
    return () => window.clearTimeout(initialization);
  }, [reload]);

  const value = useMemo<DataContextValue>(() => ({
    projects, knowledgeAreas, education, databaseInfo, loading, error, reload,
    updateMetrics: async (input) => {
      const updated = await knowledgeService.updateMetrics(input);
      setKnowledgeAreas((current) => current.map((area) => area.id === updated.id ? updated : area));
      return updated;
    },
    loadHistory: knowledgeService.history,
    saveKnowledge: async (input) => setKnowledgeAreas(await knowledgeService.save(input)),
    deleteKnowledge: async (id) => {
      const nextKnowledge = await knowledgeService.remove(id);
      const nextProjects = await projectService.list();
      setKnowledgeAreas(nextKnowledge); setProjects(nextProjects); notifyDataRelationsChanged();
    },
    saveProject: async (input) => {
      const nextProjects = await projectService.save(input);
      const nextKnowledge = await knowledgeService.list();
      setProjects(nextProjects); setKnowledgeAreas(nextKnowledge); notifyDataRelationsChanged();
    },
    deleteProject: async (id) => {
      const nextProjects = await projectService.remove(id);
      const nextKnowledge = await knowledgeService.list();
      setProjects(nextProjects); setKnowledgeAreas(nextKnowledge); notifyDataRelationsChanged();
    },
    saveEducation: async (input) => setEducation(await educationService.save(input)),
    deleteEducation: async (id) => setEducation(await educationService.remove(id)),
  }), [projects, knowledgeAreas, education, databaseInfo, loading, error, reload]);

  return <DataContext.Provider value={value}>{children}</DataContext.Provider>;
}
