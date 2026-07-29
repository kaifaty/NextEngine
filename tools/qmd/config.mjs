export const REQUIRED_QMD_VERSION = "2.5.3";
export const COLLECTION = "nextengine-docs";
export const EMBEDDING_MODEL =
  "hf:Qwen/Qwen3-Embedding-0.6B-GGUF/Qwen3-Embedding-0.6B-Q8_0.gguf";
export const RERANK_MODEL =
  "hf:ggml-org/Qwen3-Reranker-0.6B-Q8_0-GGUF/qwen3-reranker-0.6b-q8_0.gguf";
export const GENERATE_MODEL =
  "hf:tobil/qmd-query-expansion-1.7B-gguf/qmd-query-expansion-1.7B-q4_k_m.gguf";

export function qmdModelEnvironment(environment = process.env) {
  return {
    ...environment,
    QMD_EMBED_MODEL: EMBEDDING_MODEL,
    QMD_RERANK_MODEL: RERANK_MODEL,
    QMD_GENERATE_MODEL: GENERATE_MODEL,
  };
}
