/** Target user group */
export type TargetGroup = 'disabled' | 'elderly'

/** Request for text-based resume optimization */
export interface OptimizeTextRequest {
  text: string
  group: TargetGroup
}

/** Response from /api/optimize */
export interface OptimizeResponse {
  success: boolean
  data: {
    optimized_text: string
    encouragement: string
    original_text: string
  }
}

/** A single job search result */
export interface JobSearchResult {
  title: string
  url: string
  snippet: string
}

/** Request for /api/search-jobs */
export interface SearchJobsRequest {
  group: TargetGroup
  query?: string
}

/** Response from /api/search-jobs */
export interface SearchJobsResponse {
  success: boolean
  data: JobSearchResult[]
}

/** Request for /api/export-pdf */
export interface ExportPdfRequest {
  text: string
  title?: string
}

/** Possible states for the resume input area */
export type InputMethod = 'text' | 'file' | null

/** The app-level UI state */
export type AppPhase = 'input' | 'optimizing' | 'results' | 'error'

/** Layout mode */
export type FontMode = 'normal' | 'large'

/** Contrast mode */
export type ContrastMode = 'normal' | 'high'

/** Response from /api/config */
export interface ConfigResponse {
  success: boolean
  data: {
    upload_max_size_mb: number
  }
}
