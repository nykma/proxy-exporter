# proxy-exporter

一个用于从 Clash (mihomo) 代理服务的 WebSocket API 中采集流量和连接指标，并以 Prometheus 格式暴露的导出器。

## Docker 使用方法

### 构建镜像

```bash
docker build -t proxy-exporter:latest .
```

### 运行容器

```bash
docker run -d \
  --name proxy-exporter \
  -p 9898:9898 \
  -v /path/to/config.toml:/app/config.toml:ro \
  -e CONFIG_PATH=/app/config.toml \
  proxy-exporter:latest
  
curl http://127.0.0.1:9898/metrics
```

#### 使用 Docker Compose

```yaml
services:
  proxy-exporter:
    image: proxy-exporter:latest
    # build: .          # 或直接从 Dockerfile 构建
    container_name: proxy-exporter
    ports:
      - "9898:9898"
    volumes:
      - /path/to/config.toml:/app/config.toml:ro
    environment:
      - CONFIG_PATH=/app/config.toml
    restart: unless-stopped
```

### 环境变量

| 变量 | 默认值 | 说明 |
|---|---|---|
| `LISTEN_ADDRESS` | `0.0.0.0:9898` | HTTP 监听地址 |
| `CONFIG_PATH` | `./config.toml` | 配置文件路径 |

### 自定义监听地址

```bash
docker run -d \
  -p 9090:9090 \
  -e LISTEN_ADDRESS=0.0.0.0:9090 \
  -v /path/to/config.toml:/app/config.toml \
  proxy-exporter:latest
```

### 访问指标

浏览器访问 `http://localhost:9898/metrics` 即可获取 Prometheus 格式的指标数据。

## 配置方法

你课时定义多个上游代理服务：

```toml
# Will connect to: ws://10.0.12.34:9090/traffic?token=08ihv78r
[[upstream]]
name = "home-proxy"
url = "10.0.12.34"
port = 9090
token = "08ihv78r"
ssl = false

[[upstream]]
# Will connect to: wss://example.com/traffic?token=your-secret-token
name = "remote-proxy"
url = "example.com"
port = 443
token = "your-secret-token"
ssl = true
```

## 暴露的指标

### 流量总量指标

| 指标名 | 类型 | 说明 |
|---|---|---|
| `proxy_up_total_bytes` | Gauge | 上游总上传字节数 |
| `proxy_down_total_bytes` | Gauge | 上游总下载字节数 |

标签：

| Tag | 说明 |
|---|---|
| `name` | 你在 `config.toml` 里设置的 upstream name |

示例输出：

```
proxy_up_total_bytes{name="home-proxy"} 1.048576e+08
proxy_down_total_bytes{name="home-proxy"} 5.24288e+08
```

### per-connection 指标

| 指标名 | 类型 | 说明 |
|---|---|---|
| `proxy_connection_upload_bytes` | Gauge | 单个连接的上传字节数 |
| `proxy_connection_download_bytes` | Gauge | 单个连接的下载字节数 |

标签：

| Tag | 说明 |
|---|---|
| `name` | 你在 `config.toml` 里设置的 upstream name |
| `connection_id` | 连接唯一标识 |
| `network` | 网络协议（如 `tcp`、`udp`） |
| `type` | 连接类型 |
| `source_ip` | 源 IP 地址 |
| `destination_ip` | 目标 IP 地址 |
| `source_port` | 源端口 |
| `destination_port` | 目标端口 |
| `source_geo_ip` | 源 IP 的地理位置 |
| `destination_geo_ip` | 目标 IP 的地理位置 |
| `source_ip_asn` | 源 IP 的 ASN 信息 |
| `destination_ip_asn` | 目标 IP 的 ASN 信息 |
| `inbound_ip` | 入站 IP |
| `inbound_port` | 入站端口 |
| `inbound_name` | 入站名称 |
| `inbound_user` | 入站用户 |
| `host` | 目标主机名 |
| `dns_mode` | DNS 解析模式 |
| `uid` | 用户 ID |
| `process` | 进程名 |
| `process_path` | 进程路径 |
| `special_proxy` | 特殊代理规则 |
| `special_rules` | 特殊规则 |
| `remote_destination` | 远程目标地址 |
| `dscp` | DSCP 标记 |
| `sniff_host` | 嗅探主机名 |
| `rule` | 匹配的规则 |
| `rule_payload` | 规则载荷 |
| `chain` | 代理链 |
| `provider_chain` | 代理提供者链 |

示例输出：

```
proxy_connection_upload_bytes{chain="默认代理",connection_id="d2d97ec0-d27d-48c8-afc3-c08a77bd2176",destination_geo_ip="",destination_ip="192.200.0.116",destination_ip_asn="",destination_port="443",dns_mode="normal",dscp="0",host="controlplane.tailscale.com",inbound_ip="198.18.0.1",inbound_name="DEFAULT-TUN",inbound_port="33775",inbound_user="",name="home-proxy",network="tcp",process="",process_path="",provider_chain="优质服务商",remote_destination="27.44.127.138",rule="Match",rule_payload="",sniff_host="controlplane.tailscale.com",source_geo_ip="",source_ip="10.11.45.14",source_ip_asn="",source_port="57032",special_proxy="",special_rules="",type="Tun",uid="0"} 2844
```

### 注意：
1. `chain` 和 `provider_chain` 标签在原始 API 数据里是一个 List ，并且在 index 上是一一对应的。为了方便 filter 、符合 Prometheus 的最佳实践，以及尽可能保留原始数据原貌，我把每个 chain 的值都单独写成一条数据记录了。比如：
   ```json
     { "chains": [ "🇭🇰 香港 09", "全球手动" , "默认代理" , "电报消息" ] }
   ```
   会被拆分成四条 gauge 数据，它们的 `id` tag 有相同的值，gauge 值也相同，`chain` tag 分别为 `🇭🇰 香港 09` 、 `全球手动` etc. 。如果你需要统计 per ID 的连接数据，请参考以下的 PromQL: 
   ```promql
     max by (id) (proxy_connection_upload_bytes)
   ```
