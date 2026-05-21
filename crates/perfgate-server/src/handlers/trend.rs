//! Trend analysis handlers.
//!
//! Provides an endpoint to analyze metric trends for a benchmark
//! and predict budget threshold breaches.

use axum::{
    Extension, Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};

use crate::auth::{AuthContext, Scope, check_scope};
use crate::models::ApiError;
use crate::server::AppState;
use perfgate::domain::stats::trend::TrendAnalysis;

/// Query parameters for trend analysis.
#[derive(Debug, Clone, Deserialize)]
pub struct TrendQuery {
    /// Metric to analyze (e.g., wall_ms, cpu_ms, max_rss_kb).
    pub metric: String,
    /// Number of recent baselines to use for trend analysis.
    #[serde(default = "default_window")]
    pub window: u32,
    /// Budget threshold as a fraction (e.g., 0.20 for 20% regression allowed).
    #[serde(default = "default_threshold")]
    pub threshold: f64,
    /// Number of runs within which a breach is considered "critical".
    #[serde(default = "default_critical_window")]
    pub critical_window: u32,
}

fn default_window() -> u32 {
    30
}

fn default_threshold() -> f64 {
    0.20
}

fn default_critical_window() -> u32 {
    10
}

/// Response for trend analysis.
#[derive(Debug, Clone, Serialize)]
pub struct TrendResponse {
    pub project: String,
    pub benchmark: String,
    pub analysis: Option<TrendAnalysis>,
    pub values: Vec<f64>,
    pub spark: String,
    pub data_points: usize,
}

mod trend_service {
    use axum::{Json, http::StatusCode};
    use tracing::error;

    use crate::models::{ApiError, ListBaselinesQuery};
    use crate::server::AppState;
    use perfgate::domain::stats::trend::{TrendAnalysis, analyze_trend, spark_chart};
    use perfgate::domain::{TrendConfig, metric_value};
    use perfgate_types::{Direction, Metric};

    use super::{TrendQuery, TrendResponse};

    pub(super) async fn analyze(
        state: &AppState,
        project: &str,
        benchmark: &str,
        query: &TrendQuery,
    ) -> Result<TrendResponse, (StatusCode, Json<ApiError>)> {
        let metric = parse_metric(query)?;
        let baselines = fetch_baselines(state, project, benchmark, query.window).await?;

        let values = collect_metric_values(&baselines, metric);
        let spark = spark_chart(&values);
        let data_points = values.len();

        let analysis = analyze_if_possible(&values, metric, query);

        Ok(TrendResponse {
            project: project.to_owned(),
            benchmark: benchmark.to_owned(),
            analysis,
            values,
            spark,
            data_points,
        })
    }

    fn parse_metric(query: &TrendQuery) -> Result<Metric, (StatusCode, Json<ApiError>)> {
        Metric::parse_key(&query.metric).ok_or_else(|| {
            (
                StatusCode::BAD_REQUEST,
                Json(ApiError::validation(&format!(
                    "Unknown metric: {}",
                    query.metric
                ))),
            )
        })
    }

    async fn fetch_baselines(
        state: &AppState,
        project: &str,
        benchmark: &str,
        window: u32,
    ) -> Result<Vec<crate::models::BaselineSummary>, (StatusCode, Json<ApiError>)> {
        let list_query = ListBaselinesQuery {
            benchmark: Some(benchmark.to_owned()),
            include_receipt: true,
            limit: window,
            ..Default::default()
        };

        state
            .store
            .list(project, &list_query)
            .await
            .map(|response| response.baselines)
            .map_err(|error| {
                error!(error = %error, "Failed to list baselines for trend analysis");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiError::internal_error(&error.to_string())),
                )
            })
    }

    fn collect_metric_values(
        baselines: &[crate::models::BaselineSummary],
        metric: Metric,
    ) -> Vec<f64> {
        let mut entries: Vec<(chrono::DateTime<chrono::Utc>, f64)> = baselines
            .iter()
            .filter_map(|baseline| {
                let receipt = baseline.receipt.as_ref()?;
                let value = metric_value(&receipt.stats, metric)?;
                Some((baseline.created_at, value))
            })
            .collect();

        entries.sort_by_key(|(timestamp, _)| *timestamp);
        entries.into_iter().map(|(_, value)| value).collect()
    }

    fn analyze_if_possible(
        values: &[f64],
        metric: Metric,
        query: &TrendQuery,
    ) -> Option<TrendAnalysis> {
        if values.len() < 2 {
            return None;
        }

        let lower_is_better = metric.default_direction() == Direction::Lower;
        let absolute_threshold =
            compute_absolute_threshold(values[0], lower_is_better, query.threshold);
        let config = TrendConfig {
            critical_window: query.critical_window,
            ..TrendConfig::default()
        };

        analyze_trend(
            values,
            metric.as_str(),
            absolute_threshold,
            lower_is_better,
            &config,
        )
    }

    fn compute_absolute_threshold(
        baseline_value: f64,
        lower_is_better: bool,
        threshold: f64,
    ) -> f64 {
        if lower_is_better {
            baseline_value * (1.0 + threshold)
        } else {
            baseline_value * (1.0 - threshold)
        }
    }
}

/// Analyze metric trends for a benchmark.
///
/// `GET /api/v1/projects/{project}/baselines/{benchmark}/trend?metric=wall_ms&window=30`
pub async fn get_trend(
    Path((project, benchmark)): Path<(String, String)>,
    Extension(auth_ctx): Extension<AuthContext>,
    State(state): State<AppState>,
    Query(query): Query<TrendQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<ApiError>)> {
    check_scope(Some(&auth_ctx), &project, Some(&benchmark), Scope::Read)?;
    let response = trend_service::analyze(&state, &project, &benchmark, &query).await?;
    Ok(Json(response))
}
