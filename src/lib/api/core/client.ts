import { API_URL } from '$lib/utils/platform';
import { get } from 'svelte/store';
import { connectionToken } from '$lib/stores/wallet';
import { handleApiError } from './errors';

export type RequestOptions = {
  requiresAuth?: boolean;
  headers?: Record<string, string>;
};

export interface ApiResponse<T> {
  success: boolean;
  data?: T;
  error?: {
    code: ErrorCode;
    message: string;
    details?: Record<string, any>;
  };
}

export enum ErrorCode {
  // Client errors (4xx)
  BAD_REQUEST = 'BAD_REQUEST',
  UNAUTHORIZED = 'UNAUTHORIZED',
  INVALID_WALLET_SIGNATURE = 'INVALID_WALLET_SIGNATURE',
  FORBIDDEN = 'FORBIDDEN',
  NOT_FOUND = 'NOT_FOUND',
  REQ_VALIDATION_ERROR = 'REQ_VALIDATION_ERROR',
  RATE_LIMIT_EXCEEDED = 'RATE_LIMIT_EXCEEDED',
  CONFLICT = 'CONFLICT',
  PAYMENT_REQUIRED = 'PAYMENT_REQUIRED',
  GONE = 'GONE',

  // Server errors (5xx)
  INTERNAL_SERVER_ERROR = 'INTERNAL_SERVER_ERROR',
  SERVICE_UNAVAILABLE = 'SERVICE_UNAVAILABLE',

  // Domain-specific errors
  INSUFFICIENT_FUNDS = 'INSUFFICIENT_FUNDS',
  TRANSACTION_FAILED = 'TRANSACTION_FAILED',
  CHALLENGE_EXPIRED = 'CHALLENGE_EXPIRED',
  UPLOAD_INCOMPLETE = 'UPLOAD_INCOMPLETE'
}

export class ApiClient {
  private baseUrl: string;

  constructor(baseUrl = `${API_URL}/api`) {
    this.baseUrl = baseUrl;
  }

  private getAuthToken(): string {
    return get(connectionToken) || '';
  }

  private getHeaders(options: RequestOptions = {}): Headers {
    const headers = new Headers(options.headers);

    if (options.requiresAuth) {
      headers.set('x-connect-token', this.getAuthToken());
    }

    return headers;
  }

  async get<T>(
    endpoint: string,
    params?: Record<string, any>,
    options: RequestOptions = {}
  ): Promise<ApiResponse<T>> {
    const url = new URL(`${this.baseUrl}${endpoint}`);

    if (params) {
      Object.entries(params).forEach(([key, value]) => {
        if (value !== undefined && value !== null) {
          if (Array.isArray(value)) {
            url.searchParams.set(key, value.toString());
          } else {
            url.searchParams.set(key, String(value));
          }
        }
      });
    }

    const response = await fetch(url.toString(), {
      method: 'GET',
      headers: this.getHeaders(options)
    });

    if (!response.ok) {
      handleApiError(response);
    }

    return response.json() as Promise<ApiResponse<T>>;
  }

  async post<T>(
    endpoint: string,
    data?: any,
    options: RequestOptions = {}
  ): Promise<ApiResponse<T>> {
    const headers = this.getHeaders(options);

    if (data && !(data instanceof FormData)) {
      headers.set('Content-Type', 'application/json');
    }

    const response = await fetch(`${this.baseUrl}${endpoint}`, {
      method: 'POST',
      headers,
      body: data instanceof FormData ? data : JSON.stringify(data)
    });

    if (!response.ok) {
      handleApiError(response);
    }

    return response.json() as Promise<ApiResponse<T>>;
  }

  async put<T>(
    endpoint: string,
    data?: any,
    options: RequestOptions = {}
  ): Promise<ApiResponse<T>> {
    const headers = this.getHeaders(options);

    if (data && !(data instanceof FormData)) {
      headers.set('Content-Type', 'application/json');
    }

    const response = await fetch(`${this.baseUrl}${endpoint}`, {
      method: 'PUT',
      headers,
      body: data instanceof FormData ? data : JSON.stringify(data)
    });

    if (!response.ok) {
      handleApiError(response);
    }

    return response.json() as Promise<ApiResponse<T>>;
  }

  async delete<T>(endpoint: string, options: RequestOptions = {}): Promise<ApiResponse<T>> {
    const response = await fetch(`${this.baseUrl}${endpoint}`, {
      method: 'DELETE',
      headers: this.getHeaders(options)
    });

    if (!response.ok) {
      handleApiError(response);
    }

    return response.json() as Promise<ApiResponse<T>>;
  }
}

// Create and export a singleton instance
export const apiClient = new ApiClient();
