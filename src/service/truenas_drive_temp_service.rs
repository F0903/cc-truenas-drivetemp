use crate::config::PluginConfig;
use crate::polling::SharedState;
use crate::proto::device_service::v1::device_service_server::DeviceService;
use crate::proto::device_service::v1::{
    CustomFunctionOneRequest, CustomFunctionOneResponse, EnableManualFanControlRequest,
    EnableManualFanControlResponse, FixedDutyRequest, FixedDutyResponse, HealthRequest,
    HealthResponse, InitializeDeviceRequest, InitializeDeviceResponse, LcdRequest, LcdResponse,
    LightingRequest, LightingResponse, ListDevicesRequest, ListDevicesResponse,
    ResetChannelRequest, ResetChannelResponse, ShutdownRequest, ShutdownResponse,
    SpeedProfileRequest, SpeedProfileResponse, StatusRequest, StatusResponse, health_response,
};
use crate::proto::models::v1::{Device, DeviceInfo, DriverInfo, Status, TempInfo, status};
use crate::{DEVICE_NAME, SERVICE_ID, VERSION};
use std::collections::HashMap;
use std::time::Instant;
use tonic::{Request, Response, Status as GrpcStatus};

#[derive(Debug, Clone)]
pub struct TrueNasDriveTempService {
    config: PluginConfig,
    state: SharedState,
    started_at: Instant,
}

impl TrueNasDriveTempService {
    pub fn new(config: PluginConfig, state: SharedState, started_at: Instant) -> Self {
        Self {
            config,
            state,
            started_at,
        }
    }

    async fn device(&self) -> Device {
        let state = self.state.snapshot().await;
        let temps = state
            .temps
            .iter()
            .map(|temp| {
                (
                    temp.id.clone(),
                    TempInfo {
                        label: temp.label.clone(),
                        number: temp.number,
                    },
                )
            })
            .collect::<HashMap<_, _>>();

        Device {
            id: SERVICE_ID.to_string(),
            name: DEVICE_NAME.to_string(),
            uid_info: Some(format!("truenas:{}", self.config.truenas.url)),
            info: Some(DeviceInfo {
                channels: HashMap::new(),
                temps,
                lighting_speeds: Vec::new(),
                temp_min: Some(self.config.temp_min),
                temp_max: Some(self.config.temp_max),
                profile_min_length: None,
                profile_max_length: None,
                model: Some("TrueNAS drive temperature provider".to_string()),
                driver_info: Some(DriverInfo {
                    name: Some(SERVICE_ID.to_string()),
                    version: Some(VERSION.to_string()),
                    locations: vec![self.config.truenas.url.clone()],
                }),
            }),
        }
    }
}

#[tonic::async_trait]
impl DeviceService for TrueNasDriveTempService {
    async fn health(
        &self,
        _request: Request<HealthRequest>,
    ) -> Result<Response<HealthResponse>, GrpcStatus> {
        let state = self.state.snapshot().await;
        let status = if state.last_error.is_some() {
            health_response::Status::Warning
        } else {
            health_response::Status::Ok
        };

        Ok(Response::new(HealthResponse {
            name: SERVICE_ID.to_string(),
            version: VERSION.to_string(),
            status: status.into(),
            uptime_seconds: self.started_at.elapsed().as_secs(),
        }))
    }

    async fn list_devices(
        &self,
        _request: Request<ListDevicesRequest>,
    ) -> Result<Response<ListDevicesResponse>, GrpcStatus> {
        Ok(Response::new(ListDevicesResponse {
            devices: vec![self.device().await],
        }))
    }

    async fn initialize_device(
        &self,
        _request: Request<InitializeDeviceRequest>,
    ) -> Result<Response<InitializeDeviceResponse>, GrpcStatus> {
        Ok(Response::new(InitializeDeviceResponse {}))
    }

    async fn shutdown(
        &self,
        _request: Request<ShutdownRequest>,
    ) -> Result<Response<ShutdownResponse>, GrpcStatus> {
        Ok(Response::new(ShutdownResponse {}))
    }

    async fn status(
        &self,
        request: Request<StatusRequest>,
    ) -> Result<Response<StatusResponse>, GrpcStatus> {
        if request.get_ref().device_id != SERVICE_ID {
            return Err(GrpcStatus::not_found("device not found"));
        }

        let state = self.state.snapshot().await;
        let status = state
            .temps
            .iter()
            .map(|temp| Status {
                id: temp.id.clone(),
                metric: Some(status::Metric::Temp(temp.celsius)),
            })
            .collect();

        Ok(Response::new(StatusResponse { status }))
    }

    async fn reset_channel(
        &self,
        _request: Request<ResetChannelRequest>,
    ) -> Result<Response<ResetChannelResponse>, GrpcStatus> {
        Err(GrpcStatus::unimplemented("no controllable channels"))
    }

    async fn enable_manual_fan_control(
        &self,
        _request: Request<EnableManualFanControlRequest>,
    ) -> Result<Response<EnableManualFanControlResponse>, GrpcStatus> {
        Err(GrpcStatus::unimplemented("no controllable channels"))
    }

    async fn fixed_duty(
        &self,
        _request: Request<FixedDutyRequest>,
    ) -> Result<Response<FixedDutyResponse>, GrpcStatus> {
        Err(GrpcStatus::unimplemented("no controllable channels"))
    }

    async fn speed_profile(
        &self,
        _request: Request<SpeedProfileRequest>,
    ) -> Result<Response<SpeedProfileResponse>, GrpcStatus> {
        Err(GrpcStatus::unimplemented("no firmware profiles"))
    }

    async fn lighting(
        &self,
        _request: Request<LightingRequest>,
    ) -> Result<Response<LightingResponse>, GrpcStatus> {
        Err(GrpcStatus::unimplemented("no lighting channels"))
    }

    async fn lcd(
        &self,
        _request: Request<LcdRequest>,
    ) -> Result<Response<LcdResponse>, GrpcStatus> {
        Err(GrpcStatus::unimplemented("no LCD channels"))
    }

    async fn custom_function_one(
        &self,
        _request: Request<CustomFunctionOneRequest>,
    ) -> Result<Response<CustomFunctionOneResponse>, GrpcStatus> {
        Err(GrpcStatus::unimplemented("no custom function"))
    }
}
