import { useAuthStore } from '@/lib/store/auth-store';
import { config } from '@/lib/constants/config';


type HttpMethod = 'GET' | 'POST' | 'PUT' | 'DELETE' | 'PATCH';

interface RequestOptions {
    method?: HttpMethod;
    body?: any;
    headers?: HeadersInit;
    requiresAuth?: boolean;
}

class ApiClient {
    private baseUrl: string;

    constructor(baseUrl: string) {
        this.baseUrl = baseUrl;
    }

    private async request<T>(
        endpoint: string,
        options: RequestOptions = {}
    ): Promise<T> {
        const {
            method = 'GET',
            body,
            headers = {},
            requiresAuth = true,
        } = options;
        const requestHeaders: Record<string, string> = {
            'Content-Type': 'application/json',
            ...(headers as Record<string, string>), // Cast si nécessaire
        };


        if (requiresAuth) {
            const token = useAuthStore.getState().token;
            if (token) {
                requestHeaders['Authorization'] = `Bearer ${token}`;
            }
        }

        const config: RequestInit = {
            method,
            headers: requestHeaders,
        };

        if (body && method !== 'GET') {
            config.body = JSON.stringify(body);
        }

        const url = `${this.baseUrl}${endpoint}`;
        let response: Response;
        try {
            response = await fetch(url, config);
        } catch (error) {
            const message = error instanceof Error ? error.message : 'Network error';
            throw new Error(`NETWORK_ERROR: ${message}`);
        }

        if (!response.ok) {
            let errorCode = `HTTP_${response.status}`;
            try {
                const body = await response.json();
                if (body?.error?.code) {
                    errorCode = body.error.code;
                }
            } catch {
                // Keep fallback HTTP code when response is not JSON.
            }
            throw new Error(errorCode);
        }

        // Pour les DELETE qui retournent un message custom
        if (method === 'DELETE' && response.status === 204) {
            return { success: true, message: 'Deleted successfully' } as T;
        }

        return response.json();
    }

    get<T>(endpoint: string, options?: Omit<RequestOptions, 'method' | 'body'>) {
        return this.request<T>(endpoint, { ...options, method: 'GET' });
    }

    post<T>(endpoint: string, body?: any, options?: Omit<RequestOptions, 'method'>) {
        return this.request<T>(endpoint, { ...options, method: 'POST', body });
    }

    put<T>(endpoint: string, body?: any, options?: Omit<RequestOptions, 'method'>) {
        return this.request<T>(endpoint, { ...options, method: 'PUT', body });
    }

    delete<T>(endpoint: string, options?: Omit<RequestOptions, 'method' | 'body'>) {
        return this.request<T>(endpoint, { ...options, method: 'DELETE' });
    }

    patch<T>(endpoint: string, body?: any, options?: Omit<RequestOptions, 'method'>) {
        return this.request<T>(endpoint, { ...options, method: 'PATCH', body });
    }
}

export const apiClient = new ApiClient(config.apiUrl);