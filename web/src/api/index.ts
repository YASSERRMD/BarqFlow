import axios from 'axios';

const api = axios.create({
    baseURL: '/rest',
    headers: {
        'Content-Type': 'application/json',
    },
});

function redirectToLoginWithoutReload() {
    if (window.location.pathname === '/login') {
        return;
    }

    window.history.pushState({}, '', '/login');
    window.dispatchEvent(new PopStateEvent('popstate'));
}

// Add a request interceptor to include the JWT token
api.interceptors.request.use(
    (config) => {
        const token = localStorage.getItem('token');
        if (token) {
            config.headers.Authorization = `Bearer ${token}`;
        }
        return config;
    },
    (error) => {
        return Promise.reject(error);
    }
);

// Add a response interceptor to handle unauthorized errors
api.interceptors.response.use(
    (response) => {
        return response;
    },
    (error) => {
        if (error.response && error.response.status === 401) {
            localStorage.removeItem('token');
            redirectToLoginWithoutReload();
        }
        return Promise.reject(error);
    }
);

export default api;
