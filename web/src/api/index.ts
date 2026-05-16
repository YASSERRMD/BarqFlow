import axios from 'axios';
import router from '../router';

const api = axios.create({
    baseURL: '/rest',
    headers: {
        'Content-Type': 'application/json',
    },
});

// Attach JWT token to every outgoing request.
api.interceptors.request.use(
    (config) => {
        const token = localStorage.getItem('token');
        if (token) {
            config.headers.Authorization = `Bearer ${token}`;
        }
        return config;
    },
    (error) => Promise.reject(error),
);

// On 401, clear the stored token and redirect to /login via Vue Router
// so the SPA state (active route, history stack) is updated correctly.
api.interceptors.response.use(
    (response) => response,
    (error) => {
        if (error.response?.status === 401) {
            localStorage.removeItem('token');
            if (router.currentRoute.value.path !== '/login') {
                router.push('/login');
            }
        }
        return Promise.reject(error);
    },
);

export default api;
