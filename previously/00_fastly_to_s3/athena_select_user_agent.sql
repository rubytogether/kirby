SELECT user_agent
FROM access_logs
WHERE user_agent LIKE 'bundler%'
        AND ( request_path = '/api/v1/dependencies'
        OR request_path = '/versions' )