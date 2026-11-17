export interface RunOptions {
  cwd?: string;
  timeout?: number;
  maxBuffer?: number;
  env?: Record<string, string>;
}

export interface PylaError extends Error {
  status: number | null;
  stderr: string;
  stdout: string;
}

export interface AnalysisReport {
  files_analyzed: number;
  languages: Record<string, number>;
  diagnostics: Diagnostic[];
}

export interface Diagnostic {
  rule_id: string;
  severity: string;
  message: string;
  location: { file: string; start: number; end: number };
  suggestion: string | null;
}

export interface ValidationResult {
  language: string;
  tool: string;
  available: boolean;
  passed: boolean | null;
  detail: string;
}

export interface MigrationPlan {
  from: string;
  to: string;
  source: string;
  functions: string[];
  steps: string[];
  candidates: unknown[];
  semantic_fidelity: string;
}

export declare function analyze(path: string, options?: RunOptions): AnalysisReport;
export declare function migrate(path: string, to: string, options?: RunOptions & { output?: string }): MigrationPlan[] | string[];
export declare function validate(path: string, language: "rust" | "python" | "cpp", options?: RunOptions): ValidationResult;
export declare function optimize(path: string, options?: RunOptions): unknown[];
export declare function compare(caseFile: string, options?: RunOptions): { passed: boolean; cases: unknown[] };
export declare function worktree(action: "create" | "status" | "remove", repository: string, path: string, branch?: string, options?: RunOptions): unknown;
export declare const PylaError: new (message: string) => PylaError;
