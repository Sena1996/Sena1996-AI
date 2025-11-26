use super::{DomainAgentType, DomainAnalysis, Finding, Severity};
use once_cell::sync::Lazy;
use regex::Regex;

static MQTT_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(mqtt|mosquitto|pubsub|publish|subscribe)"#).expect("invalid mqtt regex")
});

static BLE_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(ble|bluetooth|characteristic|gatt|uuid)"#).expect("invalid ble regex")
});

static COAP_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(coap|constrained|observe)"#).expect("invalid coap regex")
});

static GPIO_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(gpio|pin|digital|analog|pwm|adc)"#).expect("invalid gpio regex")
});

static INTERRUPT_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(interrupt|isr|attachinterrupt|nvic|irq)"#).expect("invalid interrupt regex")
});

static SLEEP_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(sleep|deepsleep|standby|lowpower|wakeup)"#).expect("invalid sleep regex")
});

pub struct IoTAgent {
    name: String,
}

impl IoTAgent {
    pub fn new() -> Self {
        Self {
            name: "IoT Agent".to_string(),
        }
    }

    pub fn analyze(&self, command: &str, input: &str) -> DomainAnalysis {
        match command {
            "protocol" | "mqtt" | "ble" | "coap" => self.analyze_protocol(input),
            "debug" => self.debug_device(input),
            "power" => self.analyze_power(input),
            "connect" => self.analyze_connectivity(input),
            "sensor" => self.analyze_sensors(input),
            "firmware" => self.analyze_firmware(input),
            _ => self.full_analysis(input),
        }
    }

    fn analyze_protocol(&self, input: &str) -> DomainAnalysis {
        let mut findings = Vec::new();
        let input_lower = input.to_lowercase();

        if MQTT_REGEX.is_match(&input_lower) {
            findings.push(Finding {
                severity: Severity::Info,
                title: "MQTT protocol detected".to_string(),
                description: "MQTT publish/subscribe messaging pattern found".to_string(),
                location: None,
                suggestion: None,
            });

            if input_lower.contains("qos") {
                let qos_level = if input_lower.contains("qos: 2") || input_lower.contains("qos=2") {
                    findings.push(Finding {
                        severity: Severity::Success,
                        title: "QoS 2 (Exactly Once) configured".to_string(),
                        description: "Highest reliability for message delivery".to_string(),
                        location: None,
                        suggestion: Some("Note: QoS 2 has higher latency overhead".to_string()),
                    });
                    2
                } else if input_lower.contains("qos: 1") || input_lower.contains("qos=1") {
                    findings.push(Finding {
                        severity: Severity::Success,
                        title: "QoS 1 (At Least Once) configured".to_string(),
                        description: "Good balance of reliability and performance".to_string(),
                        location: None,
                        suggestion: None,
                    });
                    1
                } else {
                    findings.push(Finding {
                        severity: Severity::Warning,
                        title: "QoS 0 (At Most Once) detected".to_string(),
                        description: "Messages may be lost without acknowledgment".to_string(),
                        location: None,
                        suggestion: Some("Consider QoS 1 for important messages".to_string()),
                    });
                    0
                };
            } else {
                findings.push(Finding {
                    severity: Severity::Warning,
                    title: "No QoS level specified".to_string(),
                    description: "MQTT QoS defaults to 0, messages may be lost".to_string(),
                    location: None,
                    suggestion: Some("Explicitly set QoS level based on message importance".to_string()),
                });
            }

            if !input_lower.contains("tls") && !input_lower.contains("ssl") && !input_lower.contains("8883") {
                findings.push(Finding {
                    severity: Severity::Critical,
                    title: "MQTT without TLS detected".to_string(),
                    description: "MQTT connection may not be encrypted".to_string(),
                    location: None,
                    suggestion: Some("Use MQTTS (port 8883) with TLS encryption".to_string()),
                });
            }

            if input_lower.contains("will") || input_lower.contains("lastwill") {
                findings.push(Finding {
                    severity: Severity::Success,
                    title: "Last Will and Testament configured".to_string(),
                    description: "Device will notify on unexpected disconnect".to_string(),
                    location: None,
                    suggestion: None,
                });
            }
        }

        if BLE_REGEX.is_match(&input_lower) {
            findings.push(Finding {
                severity: Severity::Info,
                title: "Bluetooth Low Energy detected".to_string(),
                description: "BLE communication implementation found".to_string(),
                location: None,
                suggestion: None,
            });

            if input_lower.contains("notify") {
                findings.push(Finding {
                    severity: Severity::Success,
                    title: "BLE notifications enabled".to_string(),
                    description: "Characteristic notifications for real-time updates".to_string(),
                    location: None,
                    suggestion: None,
                });
            }

            if input_lower.contains("bond") || input_lower.contains("pair") {
                findings.push(Finding {
                    severity: Severity::Success,
                    title: "BLE pairing/bonding implemented".to_string(),
                    description: "Secure device pairing is configured".to_string(),
                    location: None,
                    suggestion: None,
                });
            } else {
                findings.push(Finding {
                    severity: Severity::Warning,
                    title: "No BLE bonding detected".to_string(),
                    description: "Device may accept connections from any source".to_string(),
                    location: None,
                    suggestion: Some("Implement bonding for secure connections".to_string()),
                });
            }
        }

        if COAP_REGEX.is_match(&input_lower) {
            findings.push(Finding {
                severity: Severity::Info,
                title: "CoAP protocol detected".to_string(),
                description: "Constrained Application Protocol for IoT".to_string(),
                location: None,
                suggestion: Some("Consider DTLS for secure CoAP communication".to_string()),
            });
        }

        if input_lower.contains("websocket") || input_lower.contains("ws://") {
            findings.push(Finding {
                severity: Severity::Info,
                title: "WebSocket communication detected".to_string(),
                description: "Real-time bidirectional communication".to_string(),
                location: None,
                suggestion: Some("Use WSS (WebSocket Secure) for production".to_string()),
            });
        }

        let critical_count = findings.iter().filter(|f| f.severity == Severity::Critical).count();
        let score = if critical_count > 0 { 50 } else { 80 };

        DomainAnalysis {
            agent: DomainAgentType::IoT,
            category: "Protocol Analysis".to_string(),
            findings,
            recommendations: vec![
                "Always use encrypted connections (TLS/DTLS)".to_string(),
                "Implement proper QoS levels for message reliability".to_string(),
                "Add connection retry logic with exponential backoff".to_string(),
                "Implement Last Will messages for disconnect detection".to_string(),
                "Use topic ACLs to restrict access".to_string(),
            ],
            score,
        }
    }

    fn debug_device(&self, input: &str) -> DomainAnalysis {
        let mut findings = Vec::new();
        let input_lower = input.to_lowercase();

        if input_lower.contains("error") || input_lower.contains("fail") || input_lower.contains("exception") {
            findings.push(Finding {
                severity: Severity::Warning,
                title: "Error handling detected".to_string(),
                description: "Error conditions are being handled in code".to_string(),
                location: None,
                suggestion: Some("Ensure all error paths have proper recovery".to_string()),
            });
        }

        if input_lower.contains("timeout") {
            findings.push(Finding {
                severity: Severity::Info,
                title: "Timeout handling detected".to_string(),
                description: "Code handles operation timeouts".to_string(),
                location: None,
                suggestion: Some("Consider watchdog timer for system-level recovery".to_string()),
            });
        }

        if input_lower.contains("watchdog") || input_lower.contains("wdt") {
            findings.push(Finding {
                severity: Severity::Success,
                title: "Watchdog timer implemented".to_string(),
                description: "System will recover from hangs automatically".to_string(),
                location: None,
                suggestion: None,
            });
        }

        if input_lower.contains("printf") || input_lower.contains("serial.print") || input_lower.contains("log") {
            findings.push(Finding {
                severity: Severity::Info,
                title: "Debug logging detected".to_string(),
                description: "Debug output is available for troubleshooting".to_string(),
                location: None,
                suggestion: Some("Disable verbose logging in production for power savings".to_string()),
            });
        }

        if INTERRUPT_REGEX.is_match(&input_lower) {
            findings.push(Finding {
                severity: Severity::Info,
                title: "Interrupt handlers detected".to_string(),
                description: "Hardware interrupts are being used".to_string(),
                location: None,
                suggestion: Some("Keep ISRs short, use flags for main loop processing".to_string()),
            });

            if input_lower.contains("volatile") {
                findings.push(Finding {
                    severity: Severity::Success,
                    title: "Volatile variables used with ISRs".to_string(),
                    description: "Shared variables properly marked as volatile".to_string(),
                    location: None,
                    suggestion: None,
                });
            } else {
                findings.push(Finding {
                    severity: Severity::Warning,
                    title: "Missing volatile keyword".to_string(),
                    description: "Variables shared with ISRs should be volatile".to_string(),
                    location: None,
                    suggestion: Some("Mark ISR-shared variables as volatile".to_string()),
                });
            }
        }

        if input_lower.contains("malloc") || input_lower.contains("free") || input_lower.contains("heap") {
            findings.push(Finding {
                severity: Severity::Warning,
                title: "Dynamic memory allocation detected".to_string(),
                description: "Heap usage in embedded systems can cause fragmentation".to_string(),
                location: None,
                suggestion: Some("Prefer static allocation or memory pools".to_string()),
            });
        }

        let score = 75;

        DomainAnalysis {
            agent: DomainAgentType::IoT,
            category: "Device Debugging".to_string(),
            findings,
            recommendations: vec![
                "Implement comprehensive error logging".to_string(),
                "Add watchdog timer for system recovery".to_string(),
                "Use structured logging with severity levels".to_string(),
                "Implement remote debugging capability".to_string(),
                "Add diagnostic commands for field troubleshooting".to_string(),
            ],
            score,
        }
    }

    fn analyze_power(&self, input: &str) -> DomainAnalysis {
        let mut findings = Vec::new();
        let input_lower = input.to_lowercase();

        if SLEEP_REGEX.is_match(&input_lower) {
            findings.push(Finding {
                severity: Severity::Success,
                title: "Sleep modes implemented".to_string(),
                description: "Device uses low-power sleep states".to_string(),
                location: None,
                suggestion: None,
            });

            if input_lower.contains("deepsleep") || input_lower.contains("deep_sleep") {
                findings.push(Finding {
                    severity: Severity::Success,
                    title: "Deep sleep mode used".to_string(),
                    description: "Lowest power consumption sleep state".to_string(),
                    location: None,
                    suggestion: Some("Ensure proper wake-up sources are configured".to_string()),
                });
            }
        } else {
            findings.push(Finding {
                severity: Severity::Warning,
                title: "No sleep mode detected".to_string(),
                description: "Device may consume excessive power".to_string(),
                location: None,
                suggestion: Some("Implement sleep modes for battery-powered devices".to_string()),
            });
        }

        if input_lower.contains("delay") && !input_lower.contains("sleep") {
            findings.push(Finding {
                severity: Severity::Warning,
                title: "Busy-wait delays detected".to_string(),
                description: "delay() wastes power, use sleep instead".to_string(),
                location: None,
                suggestion: Some("Replace delay() with timer-based sleep".to_string()),
            });
        }

        if input_lower.contains("adc") || input_lower.contains("analogread") {
            findings.push(Finding {
                severity: Severity::Info,
                title: "ADC usage detected".to_string(),
                description: "Analog-to-digital converter is used".to_string(),
                location: None,
                suggestion: Some("Disable ADC when not in use to save power".to_string()),
            });
        }

        if input_lower.contains("wifi") || input_lower.contains("radio") || input_lower.contains("lora") {
            findings.push(Finding {
                severity: Severity::Info,
                title: "Radio/WiFi usage detected".to_string(),
                description: "Wireless communication is a major power consumer".to_string(),
                location: None,
                suggestion: Some("Implement radio duty cycling to save power".to_string()),
            });

            if input_lower.contains("modem_sleep") || input_lower.contains("light_sleep") {
                findings.push(Finding {
                    severity: Severity::Success,
                    title: "WiFi power saving enabled".to_string(),
                    description: "WiFi modem sleep is configured".to_string(),
                    location: None,
                    suggestion: None,
                });
            }
        }

        if input_lower.contains("led") || input_lower.contains("display") || input_lower.contains("oled") {
            findings.push(Finding {
                severity: Severity::Info,
                title: "LED/Display usage detected".to_string(),
                description: "Visual output consumes power".to_string(),
                location: None,
                suggestion: Some("Implement display timeout and brightness control".to_string()),
            });
        }

        let has_sleep = SLEEP_REGEX.is_match(&input_lower);
        let score = if has_sleep { 80 } else { 55 };

        DomainAnalysis {
            agent: DomainAgentType::IoT,
            category: "Power Analysis".to_string(),
            findings,
            recommendations: vec![
                "Use deep sleep between sensor readings".to_string(),
                "Disable unused peripherals (ADC, UART, SPI)".to_string(),
                "Reduce CPU clock speed when possible".to_string(),
                "Batch transmissions to minimize radio-on time".to_string(),
                "Use hardware timers instead of busy-wait loops".to_string(),
                "Implement adaptive sampling rates".to_string(),
            ],
            score,
        }
    }

    fn analyze_connectivity(&self, input: &str) -> DomainAnalysis {
        let mut findings = Vec::new();
        let input_lower = input.to_lowercase();

        if input_lower.contains("reconnect") || input_lower.contains("retry") {
            findings.push(Finding {
                severity: Severity::Success,
                title: "Reconnection logic implemented".to_string(),
                description: "Device handles connection drops".to_string(),
                location: None,
                suggestion: None,
            });

            if input_lower.contains("exponential") || input_lower.contains("backoff") {
                findings.push(Finding {
                    severity: Severity::Success,
                    title: "Exponential backoff implemented".to_string(),
                    description: "Reconnection uses proper backoff strategy".to_string(),
                    location: None,
                    suggestion: None,
                });
            }
        } else {
            findings.push(Finding {
                severity: Severity::Warning,
                title: "No reconnection logic detected".to_string(),
                description: "Device may not recover from network issues".to_string(),
                location: None,
                suggestion: Some("Implement automatic reconnection with backoff".to_string()),
            });
        }

        if input_lower.contains("ping") || input_lower.contains("heartbeat") || input_lower.contains("keepalive") {
            findings.push(Finding {
                severity: Severity::Success,
                title: "Connection keepalive implemented".to_string(),
                description: "Device maintains connection health checks".to_string(),
                location: None,
                suggestion: None,
            });
        }

        if input_lower.contains("offline") || input_lower.contains("queue") || input_lower.contains("buffer") {
            findings.push(Finding {
                severity: Severity::Success,
                title: "Offline data handling detected".to_string(),
                description: "Device buffers data when offline".to_string(),
                location: None,
                suggestion: None,
            });
        } else {
            findings.push(Finding {
                severity: Severity::Warning,
                title: "No offline handling detected".to_string(),
                description: "Data may be lost during connectivity issues".to_string(),
                location: None,
                suggestion: Some("Implement local storage/queue for offline mode".to_string()),
            });
        }

        let score = 70;

        DomainAnalysis {
            agent: DomainAgentType::IoT,
            category: "Connectivity Analysis".to_string(),
            findings,
            recommendations: vec![
                "Implement connection state machine".to_string(),
                "Add exponential backoff for reconnections".to_string(),
                "Buffer critical data during offline periods".to_string(),
                "Implement connection quality monitoring".to_string(),
                "Add fallback communication channels".to_string(),
            ],
            score,
        }
    }

    fn analyze_sensors(&self, input: &str) -> DomainAnalysis {
        let mut findings = Vec::new();
        let input_lower = input.to_lowercase();

        let sensors = vec![
            ("temperature", "Temperature sensor"),
            ("humidity", "Humidity sensor"),
            ("pressure", "Pressure sensor"),
            ("accelerometer", "Accelerometer"),
            ("gyro", "Gyroscope"),
            ("gps", "GPS module"),
            ("ultrasonic", "Ultrasonic distance sensor"),
            ("pir", "PIR motion sensor"),
            ("light", "Light sensor"),
        ];

        for (keyword, name) in sensors {
            if input_lower.contains(keyword) {
                findings.push(Finding {
                    severity: Severity::Info,
                    title: format!("{} detected", name),
                    description: format!("{} is used in the device", name),
                    location: None,
                    suggestion: None,
                });
            }
        }

        if input_lower.contains("calibrat") {
            findings.push(Finding {
                severity: Severity::Success,
                title: "Sensor calibration detected".to_string(),
                description: "Sensors are being calibrated".to_string(),
                location: None,
                suggestion: None,
            });
        }

        if input_lower.contains("filter") || input_lower.contains("average") || input_lower.contains("median") {
            findings.push(Finding {
                severity: Severity::Success,
                title: "Data filtering implemented".to_string(),
                description: "Sensor data is being filtered/smoothed".to_string(),
                location: None,
                suggestion: None,
            });
        } else {
            findings.push(Finding {
                severity: Severity::Warning,
                title: "No data filtering detected".to_string(),
                description: "Raw sensor data may contain noise".to_string(),
                location: None,
                suggestion: Some("Implement moving average or Kalman filter".to_string()),
            });
        }

        let score = 75;

        DomainAnalysis {
            agent: DomainAgentType::IoT,
            category: "Sensor Analysis".to_string(),
            findings,
            recommendations: vec![
                "Implement sensor calibration routines".to_string(),
                "Add data filtering (moving average, Kalman)".to_string(),
                "Validate sensor readings (range checks)".to_string(),
                "Handle sensor failures gracefully".to_string(),
                "Use appropriate sampling rates".to_string(),
            ],
            score,
        }
    }

    fn analyze_firmware(&self, input: &str) -> DomainAnalysis {
        let mut findings = Vec::new();
        let input_lower = input.to_lowercase();

        if input_lower.contains("ota") || input_lower.contains("fota") || input_lower.contains("update") {
            findings.push(Finding {
                severity: Severity::Success,
                title: "OTA update capability detected".to_string(),
                description: "Device supports over-the-air updates".to_string(),
                location: None,
                suggestion: None,
            });

            if input_lower.contains("signature") || input_lower.contains("verify") || input_lower.contains("checksum") {
                findings.push(Finding {
                    severity: Severity::Success,
                    title: "Firmware verification implemented".to_string(),
                    description: "Updates are verified before installation".to_string(),
                    location: None,
                    suggestion: None,
                });
            } else {
                findings.push(Finding {
                    severity: Severity::Critical,
                    title: "No firmware verification detected".to_string(),
                    description: "OTA updates may not be verified".to_string(),
                    location: None,
                    suggestion: Some("Implement cryptographic signature verification".to_string()),
                });
            }

            if input_lower.contains("rollback") || input_lower.contains("backup") {
                findings.push(Finding {
                    severity: Severity::Success,
                    title: "Rollback capability implemented".to_string(),
                    description: "Device can recover from bad updates".to_string(),
                    location: None,
                    suggestion: None,
                });
            }
        }

        if input_lower.contains("bootloader") {
            findings.push(Finding {
                severity: Severity::Info,
                title: "Custom bootloader detected".to_string(),
                description: "Device uses custom boot process".to_string(),
                location: None,
                suggestion: Some("Ensure bootloader is secure and updatable".to_string()),
            });
        }

        if input_lower.contains("version") || input_lower.contains("firmware_version") {
            findings.push(Finding {
                severity: Severity::Success,
                title: "Version tracking implemented".to_string(),
                description: "Firmware version is tracked".to_string(),
                location: None,
                suggestion: None,
            });
        }

        let score = 70;

        DomainAnalysis {
            agent: DomainAgentType::IoT,
            category: "Firmware Analysis".to_string(),
            findings,
            recommendations: vec![
                "Implement secure OTA update mechanism".to_string(),
                "Add cryptographic signature verification".to_string(),
                "Maintain dual firmware partitions for rollback".to_string(),
                "Implement version tracking and reporting".to_string(),
                "Add secure boot chain".to_string(),
            ],
            score,
        }
    }

    fn full_analysis(&self, input: &str) -> DomainAnalysis {
        let mut all_findings = Vec::new();

        let analyses = vec![
            self.analyze_protocol(input),
            self.debug_device(input),
            self.analyze_power(input),
            self.analyze_connectivity(input),
            self.analyze_sensors(input),
            self.analyze_firmware(input),
        ];

        let mut total_score = 0u32;
        for analysis in &analyses {
            all_findings.extend(analysis.findings.clone());
            total_score += analysis.score as u32;
        }

        let avg_score = (total_score / analyses.len() as u32) as u8;

        DomainAnalysis {
            agent: DomainAgentType::IoT,
            category: "Full IoT Analysis".to_string(),
            findings: all_findings,
            recommendations: vec![
                "Implement comprehensive error handling".to_string(),
                "Add power management for battery operation".to_string(),
                "Secure all communication channels".to_string(),
                "Implement OTA update capability".to_string(),
                "Add device provisioning workflow".to_string(),
                "Implement remote diagnostics".to_string(),
            ],
            score: avg_score,
        }
    }
}

impl Default for IoTAgent {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_iot_agent_creation() {
        let agent = IoTAgent::new();
        assert_eq!(agent.name, "IoT Agent");
    }

    #[test]
    fn test_mqtt_analysis() {
        let agent = IoTAgent::new();
        let code = r#"
            client.publish("sensors/temp", payload, { qos: 1 });
            client.subscribe("commands/#");
        "#;
        let result = agent.analyze("protocol", code);
        assert!(result.findings.iter().any(|f| f.title.contains("MQTT")));
    }

    #[test]
    fn test_power_analysis() {
        let agent = IoTAgent::new();
        let code = r#"
            esp_deep_sleep_start();
            wifi_set_sleep_type(MODEM_SLEEP_T);
        "#;
        let result = agent.analyze("power", code);
        assert!(result.findings.iter().any(|f| f.title.contains("sleep")));
    }

    #[test]
    fn test_ble_analysis() {
        let agent = IoTAgent::new();
        let code = r#"
            BLECharacteristic* pCharacteristic = pService->createCharacteristic(
                CHARACTERISTIC_UUID,
                BLECharacteristic::PROPERTY_NOTIFY
            );
        "#;
        let result = agent.analyze("protocol", code);
        assert!(result.findings.iter().any(|f| f.title.contains("Bluetooth")));
    }
}
