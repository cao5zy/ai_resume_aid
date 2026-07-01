import axios from 'axios'
import type {
  OptimizeTextRequest,
  OptimizeResponse,
  SearchJobsRequest,
  SearchJobsResponse,
  ConfigResponse,
  TargetGroup,
} from '@/types'

const api = axios.create({
  baseURL: import.meta.env.VITE_API_BASE_URL || '/api',
  timeout: 60000,
  headers: {
    'Content-Type': 'application/json',
  },
})

/**
 * Optimize resume text.
 */
export async function optimizeResumeText(
  text: string,
  group: TargetGroup
): Promise<OptimizeResponse> {
  const body: OptimizeTextRequest = { text, group }
  const { data } = await api.post<OptimizeResponse>('/optimize', body)
  return data
}

/**
 * Optimize resume from a PDF file.
 */
export async function optimizeResumeFile(
  file: File,
  group: TargetGroup
): Promise<OptimizeResponse> {
  const formData = new FormData()
  formData.append('file', file)
  formData.append('group', group)

  const { data } = await api.post<OptimizeResponse>('/optimize', formData, {
    headers: { 'Content-Type': 'multipart/form-data' },
    timeout: 300000, // 5 minutes for large PDF uploads
  })
  return data
}

/**
 * Search inclusive job listings.
 */
export async function searchJobs(
  group: TargetGroup,
  query?: string
): Promise<SearchJobsResponse> {
  const body: SearchJobsRequest = { group, query }
  const { data } = await api.post<SearchJobsResponse>('/search-jobs', body)
  return data
}

/**
 * Export optimized resume as PDF blob.
 */
export async function exportPdf(text: string, title?: string): Promise<Blob> {
  const { data } = await api.post(
    '/export-pdf',
    { text, title },
    { responseType: 'blob' }
  )
  return data
}

/**
 * Fetch runtime config from the backend.
 */
export async function fetchConfig(): Promise<ConfigResponse> {
  const { data } = await api.get<ConfigResponse>('/config')
  return data
}
