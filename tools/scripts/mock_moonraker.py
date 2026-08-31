#!/usr/bin/env python3
"""
Mock Moonraker HTTP & WebSocket Server for KlipperScreen GUI capture.
Serves full printer objects, state, endpoints on port 7125.
"""

import asyncio
import json
from aiohttp import web

PRINTER_OBJECTS = {
    "objects": {
        "webhooks": {"state": "ready", "state_message": "Printer is ready"},
        "print_stats": {
            "filename": "voron_cube_v2.gcode",
            "total_duration": 2840,
            "print_duration": 1820,
            "filament_used": 1420.5,
            "state": "printing",
            "message": "",
            "info": {"current_layer": 78, "total_layer": 120}
        },
        "display_status": {"progress": 0.64, "message": "Printing layer 78/120"},
        "virtual_sdcard": {"progress": 0.64, "is_active": True, "file_position": 624000},
        "extruder": {
            "temperature": 215.5,
            "target": 220.0,
            "power": 0.45,
            "can_extrude": True,
            "pressure_advance": 0.045,
            "smooth_time": 0.04
        },
        "heater_bed": {
            "temperature": 60.2,
            "target": 65.0,
            "power": 0.60
        },
        "temperature_sensor chamber": {
            "temperature": 45.0,
            "measured_min_temp": 20.0,
            "measured_max_temp": 60.0
        },
        "fan": {"speed": 1.0},
        "heater_fan hotend_fan": {"speed": 1.0},
        "controller_fan mcu_fan": {"speed": 0.5},
        "gcode_move": {
            "speed_factor": 1.05,
            "extrude_factor": 1.0,
            "speed": 150.0,
            "homing_origin": [0.0, 0.0, 0.0, 0.0],
            "position": [125.4, 100.0, 14.8, 42.5],
            "gcode_position": [125.4, 100.0, 14.8, 42.5]
        },
        "toolhead": {
            "homed_axes": "xyz",
            "print_time": 1820.0,
            "estimated_print_time": 2840.0,
            "max_velocity": 300.0,
            "max_accel": 3000.0,
            "max_accel_to_decel": 1500.0,
            "square_corner_velocity": 5.0,
            "axis_minimum": [0.0, 0.0, 0.0, 0.0],
            "axis_maximum": [250.0, 250.0, 250.0, 0.0],
            "position": [125.4, 100.0, 14.8, 42.5],
            "status": "Ready",
            "extruder": "extruder"
        },
        "bed_mesh": {
            "profile_name": "default",
            "mesh_min": [25.0, 25.0],
            "mesh_max": [225.0, 225.0],
            "probed_matrix": [[0.01, 0.02, 0.0, -0.02, -0.04], [0.03, 0.05, 0.02, 0.0, -0.02]],
            "mesh_matrix": [[0.01, 0.02, 0.0, -0.02, -0.04], [0.03, 0.05, 0.02, 0.0, -0.02]]
        },
        "configfile": {
            "config": {
                "extruder": {"nozzle_diameter": 0.4, "filament_diameter": 1.75},
                "heater_bed": {"min_temp": 0, "max_temp": 120},
                "bed_mesh": {},
                "screws_tilt_adjust": {},
                "input_shaper": {},
                "gcode_macro PRINT_START": {},
                "gcode_macro PRINT_END": {}
            },
            "settings": {
                "extruder": {"max_temp": 300, "min_extrude_temp": 170},
                "heater_bed": {"max_temp": 120}
            }
        }
    }
}

async def server_info(request):
    return web.json_response({
        "result": {
            "klippy_state": "ready",
            "klippy_connected": True,
            "components": ["database", "file_manager", "job_queue", "update_manager", "power", "spoolman"],
            "failed_components": [],
            "registered_directories": ["gcodes", "config"],
            "warnings": [],
            "websocket_count": 1,
            "moonraker_version": "v0.8.0-rklipp",
            "api_version": [1, 0, 0]
        }
    })

async def printer_info(request):
    return web.json_response({
        "result": {
            "state": "ready",
            "state_message": "Printer is ready",
            "hostname": "mks-skipr-rklipp",
            "software_version": "v0.12.0-rklipp",
            "cpu_info": "ARM Cortex-A53 4 cores"
        }
    })

async def printer_objects_list(request):
    return web.json_response({
        "result": {
            "objects": list(PRINTER_OBJECTS["objects"].keys())
        }
    })

async def printer_objects_query(request):
    return web.json_response({
        "result": {
            "status": PRINTER_OBJECTS["objects"],
            "eventtime": 1820.0
        }
    })

async def printer_objects_subscribe(request):
    return web.json_response({
        "result": {
            "status": PRINTER_OBJECTS["objects"],
            "eventtime": 1820.0
        }
    })

async def server_files_list(request):
    return web.json_response({
        "result": [
            {
                "path": "voron_cube_v2.gcode",
                "size": 1245088,
                "modified": 1690000000,
                "uuid": "abc-123",
                "estimated_time": 2840,
                "filament_total": 1420.5
            },
            {
                "path": "3dbenchy_pla.gcode",
                "size": 4500120,
                "modified": 1690005000,
                "uuid": "def-456",
                "estimated_time": 3600,
                "filament_total": 2100.0
            }
        ]
    })

async def websocket_handler(request):
    ws = web.WebSocketResponse()
    await ws.prepare(request)
    print("WebSocket client connected")

    async for msg in ws:
        if msg.type == web.WSMsgType.TEXT:
            try:
                data = json.loads(msg.data)
                req_id = data.get("id")
                method = data.get("method")
                params = data.get("params", {})

                if method == "server.info":
                    resp = {"jsonrpc": "2.0", "id": req_id, "result": {
                        "klippy_state": "ready",
                        "klippy_connected": True,
                        "components": ["database", "file_manager", "power", "spoolman"],
                        "moonraker_version": "v0.8.0-rklipp",
                        "api_version": [1, 0, 0]
                    }}
                elif method == "printer.info":
                    resp = {"jsonrpc": "2.0", "id": req_id, "result": {
                        "state": "ready", "state_message": "Printer is ready", "hostname": "mks-skipr-rklipp"
                    }}
                elif method == "printer.objects.list":
                    resp = {"jsonrpc": "2.0", "id": req_id, "result": {"objects": list(PRINTER_OBJECTS["objects"].keys())}}
                elif method == "printer.objects.query" or method == "printer.objects.subscribe":
                    resp = {"jsonrpc": "2.0", "id": req_id, "result": {"status": PRINTER_OBJECTS["objects"], "eventtime": 1820.0}}
                elif method == "server.files.list":
                    resp = {"jsonrpc": "2.0", "id": req_id, "result": [
                        {"path": "voron_cube_v2.gcode", "size": 1245088, "modified": 1690000000},
                        {"path": "3dbenchy_pla.gcode", "size": 4500120, "modified": 1690005000}
                    ]}
                else:
                    resp = {"jsonrpc": "2.0", "id": req_id, "result": "ok"}

                await ws.send_json(resp)
            except Exception as e:
                print("Error handling WS message:", e)

    return ws

def create_app():
    app = web.Application()
    app.router.add_get('/server/info', server_info)
    app.router.add_get('/printer/info', printer_info)
    app.router.add_get('/printer/objects/list', printer_objects_list)
    app.router.add_get('/printer/objects/query', printer_objects_query)
    app.router.add_get('/printer/objects/subscribe', printer_objects_subscribe)
    app.router.add_get('/server/files/list', server_files_list)
    app.router.add_get('/websocket', websocket_handler)
    return app

if __name__ == '__main__':
    app = create_app()
    web.run_app(app, host='127.0.0.1', port=7125)
