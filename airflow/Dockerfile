# Use the official Airflow image with Python 3.11
FROM apache/airflow:2.9.3-python3.11

USER root

# Install system dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    gcc \
    g++ \
    libpq-dev \
    python3-dev \
    && rm -rf /var/lib/apt/lists/*

# Switch to airflow user
USER airflow

# Set environment variables
ENV AIRFLOW_HOME=/opt/airflow
ENV AIRFLOW__CORE__EXECUTOR=LocalExecutor
ENV AIRFLOW__CORE__SQL_ALCHEMY_CONN=postgresql+psycopg2://airflow:airflow@postgres/airflow
ENV AIRFLOW__CORE__LOAD_EXAMPLES=false
ENV AIRFLOW__WEBSERVER__WORKER_REFRESH_INTERVAL=1800

# Install Python dependencies
COPY --chown=airflow:root requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt --user && \
    rm -f requirements.txt

# Copy DAGs and plugins
COPY --chown=airflow:root dags/ ${AIRFLOW_HOME}/dags/
COPY --chown=airflow:root plugins/ ${AIRFLOW_HOME}/plugins/

# Create necessary directories
RUN mkdir -p ${AIRFLOW_HOME}/logs \
    && chown -R airflow:root ${AIRFLOW_HOME}

# Health check
HEALTHCHECK --interval=30s --timeout=30s --start-period=5s --retries=3 \
    CMD airflow db check || exit 1

# Default command
CMD ["airflow", "webserver"]

