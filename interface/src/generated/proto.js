/*eslint-disable block-scoped-var, id-length, no-control-regex, no-magic-numbers, no-prototype-builtins, no-redeclare, no-shadow, no-var, sort-vars*/
import * as $protobuf from "protobufjs/minimal";

// Common aliases
const $Reader = $protobuf.Reader, $Writer = $protobuf.Writer, $util = $protobuf.util;

// Exported root namespace
const $root = $protobuf.roots["default"] || ($protobuf.roots["default"] = {});

export const lifelog = $root.lifelog = (() => {

    /**
     * Namespace lifelog.
     * @exports lifelog
     * @namespace
     */
    const lifelog = {};

    lifelog.LifelogService = (function() {

        /**
         * Constructs a new LifelogService service.
         * @memberof lifelog
         * @classdesc Represents a LifelogService
         * @extends $protobuf.rpc.Service
         * @constructor
         * @param {$protobuf.RPCImpl} rpcImpl RPC implementation
         * @param {boolean} [requestDelimited=false] Whether requests are length-delimited
         * @param {boolean} [responseDelimited=false] Whether responses are length-delimited
         */
        function LifelogService(rpcImpl, requestDelimited, responseDelimited) {
            $protobuf.rpc.Service.call(this, rpcImpl, requestDelimited, responseDelimited);
        }

        (LifelogService.prototype = Object.create($protobuf.rpc.Service.prototype)).constructor = LifelogService;

        /**
         * Creates new LifelogService service using the specified rpc implementation.
         * @function create
         * @memberof lifelog.LifelogService
         * @static
         * @param {$protobuf.RPCImpl} rpcImpl RPC implementation
         * @param {boolean} [requestDelimited=false] Whether requests are length-delimited
         * @param {boolean} [responseDelimited=false] Whether responses are length-delimited
         * @returns {LifelogService} RPC service. Useful where requests and/or responses are streamed.
         */
        LifelogService.create = function create(rpcImpl, requestDelimited, responseDelimited) {
            return new this(rpcImpl, requestDelimited, responseDelimited);
        };

        /**
         * Callback as used by {@link lifelog.LifelogService#search}.
         * @memberof lifelog.LifelogService
         * @typedef SearchCallback
         * @type {function}
         * @param {Error|null} error Error, if any
         * @param {lifelog.SearchResponse} [response] SearchResponse
         */

        /**
         * Calls Search.
         * @function search
         * @memberof lifelog.LifelogService
         * @instance
         * @param {lifelog.ISearchRequest} request SearchRequest message or plain object
         * @param {lifelog.LifelogService.SearchCallback} callback Node-style callback called with the error, if any, and SearchResponse
         * @returns {undefined}
         * @variation 1
         */
        Object.defineProperty(LifelogService.prototype.search = function search(request, callback) {
            return this.rpcCall(search, $root.lifelog.SearchRequest, $root.lifelog.SearchResponse, request, callback);
        }, "name", { value: "Search" });

        /**
         * Calls Search.
         * @function search
         * @memberof lifelog.LifelogService
         * @instance
         * @param {lifelog.ISearchRequest} request SearchRequest message or plain object
         * @returns {Promise<lifelog.SearchResponse>} Promise
         * @variation 2
         */

        /**
         * Callback as used by {@link lifelog.LifelogService#getScreenshots}.
         * @memberof lifelog.LifelogService
         * @typedef GetScreenshotsCallback
         * @type {function}
         * @param {Error|null} error Error, if any
         * @param {lifelog.ScreenshotData} [response] ScreenshotData
         */

        /**
         * Calls GetScreenshots.
         * @function getScreenshots
         * @memberof lifelog.LifelogService
         * @instance
         * @param {lifelog.ITimeRangeRequest} request TimeRangeRequest message or plain object
         * @param {lifelog.LifelogService.GetScreenshotsCallback} callback Node-style callback called with the error, if any, and ScreenshotData
         * @returns {undefined}
         * @variation 1
         */
        Object.defineProperty(LifelogService.prototype.getScreenshots = function getScreenshots(request, callback) {
            return this.rpcCall(getScreenshots, $root.lifelog.TimeRangeRequest, $root.lifelog.ScreenshotData, request, callback);
        }, "name", { value: "GetScreenshots" });

        /**
         * Calls GetScreenshots.
         * @function getScreenshots
         * @memberof lifelog.LifelogService
         * @instance
         * @param {lifelog.ITimeRangeRequest} request TimeRangeRequest message or plain object
         * @returns {Promise<lifelog.ScreenshotData>} Promise
         * @variation 2
         */

        /**
         * Callback as used by {@link lifelog.LifelogService#getProcesses}.
         * @memberof lifelog.LifelogService
         * @typedef GetProcessesCallback
         * @type {function}
         * @param {Error|null} error Error, if any
         * @param {lifelog.ProcessData} [response] ProcessData
         */

        /**
         * Calls GetProcesses.
         * @function getProcesses
         * @memberof lifelog.LifelogService
         * @instance
         * @param {lifelog.ITimeRangeRequest} request TimeRangeRequest message or plain object
         * @param {lifelog.LifelogService.GetProcessesCallback} callback Node-style callback called with the error, if any, and ProcessData
         * @returns {undefined}
         * @variation 1
         */
        Object.defineProperty(LifelogService.prototype.getProcesses = function getProcesses(request, callback) {
            return this.rpcCall(getProcesses, $root.lifelog.TimeRangeRequest, $root.lifelog.ProcessData, request, callback);
        }, "name", { value: "GetProcesses" });

        /**
         * Calls GetProcesses.
         * @function getProcesses
         * @memberof lifelog.LifelogService
         * @instance
         * @param {lifelog.ITimeRangeRequest} request TimeRangeRequest message or plain object
         * @returns {Promise<lifelog.ProcessData>} Promise
         * @variation 2
         */

        /**
         * Callback as used by {@link lifelog.LifelogService#getCameraFrames}.
         * @memberof lifelog.LifelogService
         * @typedef GetCameraFramesCallback
         * @type {function}
         * @param {Error|null} error Error, if any
         * @param {lifelog.CameraFrameData} [response] CameraFrameData
         */

        /**
         * Calls GetCameraFrames.
         * @function getCameraFrames
         * @memberof lifelog.LifelogService
         * @instance
         * @param {lifelog.ITimeRangeRequest} request TimeRangeRequest message or plain object
         * @param {lifelog.LifelogService.GetCameraFramesCallback} callback Node-style callback called with the error, if any, and CameraFrameData
         * @returns {undefined}
         * @variation 1
         */
        Object.defineProperty(LifelogService.prototype.getCameraFrames = function getCameraFrames(request, callback) {
            return this.rpcCall(getCameraFrames, $root.lifelog.TimeRangeRequest, $root.lifelog.CameraFrameData, request, callback);
        }, "name", { value: "GetCameraFrames" });

        /**
         * Calls GetCameraFrames.
         * @function getCameraFrames
         * @memberof lifelog.LifelogService
         * @instance
         * @param {lifelog.ITimeRangeRequest} request TimeRangeRequest message or plain object
         * @returns {Promise<lifelog.CameraFrameData>} Promise
         * @variation 2
         */

        /**
         * Callback as used by {@link lifelog.LifelogService#getActivitySummary}.
         * @memberof lifelog.LifelogService
         * @typedef GetActivitySummaryCallback
         * @type {function}
         * @param {Error|null} error Error, if any
         * @param {lifelog.ActivitySummary} [response] ActivitySummary
         */

        /**
         * Calls GetActivitySummary.
         * @function getActivitySummary
         * @memberof lifelog.LifelogService
         * @instance
         * @param {lifelog.ITimeRangeRequest} request TimeRangeRequest message or plain object
         * @param {lifelog.LifelogService.GetActivitySummaryCallback} callback Node-style callback called with the error, if any, and ActivitySummary
         * @returns {undefined}
         * @variation 1
         */
        Object.defineProperty(LifelogService.prototype.getActivitySummary = function getActivitySummary(request, callback) {
            return this.rpcCall(getActivitySummary, $root.lifelog.TimeRangeRequest, $root.lifelog.ActivitySummary, request, callback);
        }, "name", { value: "GetActivitySummary" });

        /**
         * Calls GetActivitySummary.
         * @function getActivitySummary
         * @memberof lifelog.LifelogService
         * @instance
         * @param {lifelog.ITimeRangeRequest} request TimeRangeRequest message or plain object
         * @returns {Promise<lifelog.ActivitySummary>} Promise
         * @variation 2
         */

        /**
         * Callback as used by {@link lifelog.LifelogService#getProcessStats}.
         * @memberof lifelog.LifelogService
         * @typedef GetProcessStatsCallback
         * @type {function}
         * @param {Error|null} error Error, if any
         * @param {lifelog.ProcessStatsResponse} [response] ProcessStatsResponse
         */

        /**
         * Calls GetProcessStats.
         * @function getProcessStats
         * @memberof lifelog.LifelogService
         * @instance
         * @param {lifelog.IProcessStatsRequest} request ProcessStatsRequest message or plain object
         * @param {lifelog.LifelogService.GetProcessStatsCallback} callback Node-style callback called with the error, if any, and ProcessStatsResponse
         * @returns {undefined}
         * @variation 1
         */
        Object.defineProperty(LifelogService.prototype.getProcessStats = function getProcessStats(request, callback) {
            return this.rpcCall(getProcessStats, $root.lifelog.ProcessStatsRequest, $root.lifelog.ProcessStatsResponse, request, callback);
        }, "name", { value: "GetProcessStats" });

        /**
         * Calls GetProcessStats.
         * @function getProcessStats
         * @memberof lifelog.LifelogService
         * @instance
         * @param {lifelog.IProcessStatsRequest} request ProcessStatsRequest message or plain object
         * @returns {Promise<lifelog.ProcessStatsResponse>} Promise
         * @variation 2
         */

        /**
         * Callback as used by {@link lifelog.LifelogService#login}.
         * @memberof lifelog.LifelogService
         * @typedef LoginCallback
         * @type {function}
         * @param {Error|null} error Error, if any
         * @param {lifelog.LoginResponse} [response] LoginResponse
         */

        /**
         * Calls Login.
         * @function login
         * @memberof lifelog.LifelogService
         * @instance
         * @param {lifelog.ILoginRequest} request LoginRequest message or plain object
         * @param {lifelog.LifelogService.LoginCallback} callback Node-style callback called with the error, if any, and LoginResponse
         * @returns {undefined}
         * @variation 1
         */
        Object.defineProperty(LifelogService.prototype.login = function login(request, callback) {
            return this.rpcCall(login, $root.lifelog.LoginRequest, $root.lifelog.LoginResponse, request, callback);
        }, "name", { value: "Login" });

        /**
         * Calls Login.
         * @function login
         * @memberof lifelog.LifelogService
         * @instance
         * @param {lifelog.ILoginRequest} request LoginRequest message or plain object
         * @returns {Promise<lifelog.LoginResponse>} Promise
         * @variation 2
         */

        /**
         * Callback as used by {@link lifelog.LifelogService#register}.
         * @memberof lifelog.LifelogService
         * @typedef RegisterCallback
         * @type {function}
         * @param {Error|null} error Error, if any
         * @param {lifelog.RegisterResponse} [response] RegisterResponse
         */

        /**
         * Calls Register.
         * @function register
         * @memberof lifelog.LifelogService
         * @instance
         * @param {lifelog.IRegisterRequest} request RegisterRequest message or plain object
         * @param {lifelog.LifelogService.RegisterCallback} callback Node-style callback called with the error, if any, and RegisterResponse
         * @returns {undefined}
         * @variation 1
         */
        Object.defineProperty(LifelogService.prototype.register = function register(request, callback) {
            return this.rpcCall(register, $root.lifelog.RegisterRequest, $root.lifelog.RegisterResponse, request, callback);
        }, "name", { value: "Register" });

        /**
         * Calls Register.
         * @function register
         * @memberof lifelog.LifelogService
         * @instance
         * @param {lifelog.IRegisterRequest} request RegisterRequest message or plain object
         * @returns {Promise<lifelog.RegisterResponse>} Promise
         * @variation 2
         */

        /**
         * Callback as used by {@link lifelog.LifelogService#getUserProfile}.
         * @memberof lifelog.LifelogService
         * @typedef GetUserProfileCallback
         * @type {function}
         * @param {Error|null} error Error, if any
         * @param {lifelog.UserProfile} [response] UserProfile
         */

        /**
         * Calls GetUserProfile.
         * @function getUserProfile
         * @memberof lifelog.LifelogService
         * @instance
         * @param {lifelog.IUserRequest} request UserRequest message or plain object
         * @param {lifelog.LifelogService.GetUserProfileCallback} callback Node-style callback called with the error, if any, and UserProfile
         * @returns {undefined}
         * @variation 1
         */
        Object.defineProperty(LifelogService.prototype.getUserProfile = function getUserProfile(request, callback) {
            return this.rpcCall(getUserProfile, $root.lifelog.UserRequest, $root.lifelog.UserProfile, request, callback);
        }, "name", { value: "GetUserProfile" });

        /**
         * Calls GetUserProfile.
         * @function getUserProfile
         * @memberof lifelog.LifelogService
         * @instance
         * @param {lifelog.IUserRequest} request UserRequest message or plain object
         * @returns {Promise<lifelog.UserProfile>} Promise
         * @variation 2
         */

        /**
         * Callback as used by {@link lifelog.LifelogService#getLoggerStatus}.
         * @memberof lifelog.LifelogService
         * @typedef GetLoggerStatusCallback
         * @type {function}
         * @param {Error|null} error Error, if any
         * @param {lifelog.LoggerStatusResponse} [response] LoggerStatusResponse
         */

        /**
         * Calls GetLoggerStatus.
         * @function getLoggerStatus
         * @memberof lifelog.LifelogService
         * @instance
         * @param {lifelog.ILoggerStatusRequest} request LoggerStatusRequest message or plain object
         * @param {lifelog.LifelogService.GetLoggerStatusCallback} callback Node-style callback called with the error, if any, and LoggerStatusResponse
         * @returns {undefined}
         * @variation 1
         */
        Object.defineProperty(LifelogService.prototype.getLoggerStatus = function getLoggerStatus(request, callback) {
            return this.rpcCall(getLoggerStatus, $root.lifelog.LoggerStatusRequest, $root.lifelog.LoggerStatusResponse, request, callback);
        }, "name", { value: "GetLoggerStatus" });

        /**
         * Calls GetLoggerStatus.
         * @function getLoggerStatus
         * @memberof lifelog.LifelogService
         * @instance
         * @param {lifelog.ILoggerStatusRequest} request LoggerStatusRequest message or plain object
         * @returns {Promise<lifelog.LoggerStatusResponse>} Promise
         * @variation 2
         */

        /**
         * Callback as used by {@link lifelog.LifelogService#toggleLogger}.
         * @memberof lifelog.LifelogService
         * @typedef ToggleLoggerCallback
         * @type {function}
         * @param {Error|null} error Error, if any
         * @param {lifelog.ToggleLoggerResponse} [response] ToggleLoggerResponse
         */

        /**
         * Calls ToggleLogger.
         * @function toggleLogger
         * @memberof lifelog.LifelogService
         * @instance
         * @param {lifelog.IToggleLoggerRequest} request ToggleLoggerRequest message or plain object
         * @param {lifelog.LifelogService.ToggleLoggerCallback} callback Node-style callback called with the error, if any, and ToggleLoggerResponse
         * @returns {undefined}
         * @variation 1
         */
        Object.defineProperty(LifelogService.prototype.toggleLogger = function toggleLogger(request, callback) {
            return this.rpcCall(toggleLogger, $root.lifelog.ToggleLoggerRequest, $root.lifelog.ToggleLoggerResponse, request, callback);
        }, "name", { value: "ToggleLogger" });

        /**
         * Calls ToggleLogger.
         * @function toggleLogger
         * @memberof lifelog.LifelogService
         * @instance
         * @param {lifelog.IToggleLoggerRequest} request ToggleLoggerRequest message or plain object
         * @returns {Promise<lifelog.ToggleLoggerResponse>} Promise
         * @variation 2
         */

        /**
         * Callback as used by {@link lifelog.LifelogService#takeSnapshot}.
         * @memberof lifelog.LifelogService
         * @typedef TakeSnapshotCallback
         * @type {function}
         * @param {Error|null} error Error, if any
         * @param {lifelog.SnapshotResponse} [response] SnapshotResponse
         */

        /**
         * Calls TakeSnapshot.
         * @function takeSnapshot
         * @memberof lifelog.LifelogService
         * @instance
         * @param {lifelog.ISnapshotRequest} request SnapshotRequest message or plain object
         * @param {lifelog.LifelogService.TakeSnapshotCallback} callback Node-style callback called with the error, if any, and SnapshotResponse
         * @returns {undefined}
         * @variation 1
         */
        Object.defineProperty(LifelogService.prototype.takeSnapshot = function takeSnapshot(request, callback) {
            return this.rpcCall(takeSnapshot, $root.lifelog.SnapshotRequest, $root.lifelog.SnapshotResponse, request, callback);
        }, "name", { value: "TakeSnapshot" });

        /**
         * Calls TakeSnapshot.
         * @function takeSnapshot
         * @memberof lifelog.LifelogService
         * @instance
         * @param {lifelog.ISnapshotRequest} request SnapshotRequest message or plain object
         * @returns {Promise<lifelog.SnapshotResponse>} Promise
         * @variation 2
         */

        return LifelogService;
    })();

    lifelog.TimeRangeRequest = (function() {

        /**
         * Properties of a TimeRangeRequest.
         * @memberof lifelog
         * @interface ITimeRangeRequest
         * @property {string|null} [startTime] TimeRangeRequest startTime
         * @property {string|null} [endTime] TimeRangeRequest endTime
         * @property {number|null} [limit] TimeRangeRequest limit
         * @property {number|null} [offset] TimeRangeRequest offset
         */

        /**
         * Constructs a new TimeRangeRequest.
         * @memberof lifelog
         * @classdesc Represents a TimeRangeRequest.
         * @implements ITimeRangeRequest
         * @constructor
         * @param {lifelog.ITimeRangeRequest=} [properties] Properties to set
         */
        function TimeRangeRequest(properties) {
            if (properties)
                for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                    if (properties[keys[i]] != null)
                        this[keys[i]] = properties[keys[i]];
        }

        /**
         * TimeRangeRequest startTime.
         * @member {string} startTime
         * @memberof lifelog.TimeRangeRequest
         * @instance
         */
        TimeRangeRequest.prototype.startTime = "";

        /**
         * TimeRangeRequest endTime.
         * @member {string} endTime
         * @memberof lifelog.TimeRangeRequest
         * @instance
         */
        TimeRangeRequest.prototype.endTime = "";

        /**
         * TimeRangeRequest limit.
         * @member {number} limit
         * @memberof lifelog.TimeRangeRequest
         * @instance
         */
        TimeRangeRequest.prototype.limit = 0;

        /**
         * TimeRangeRequest offset.
         * @member {number} offset
         * @memberof lifelog.TimeRangeRequest
         * @instance
         */
        TimeRangeRequest.prototype.offset = 0;

        /**
         * Creates a new TimeRangeRequest instance using the specified properties.
         * @function create
         * @memberof lifelog.TimeRangeRequest
         * @static
         * @param {lifelog.ITimeRangeRequest=} [properties] Properties to set
         * @returns {lifelog.TimeRangeRequest} TimeRangeRequest instance
         */
        TimeRangeRequest.create = function create(properties) {
            return new TimeRangeRequest(properties);
        };

        /**
         * Encodes the specified TimeRangeRequest message. Does not implicitly {@link lifelog.TimeRangeRequest.verify|verify} messages.
         * @function encode
         * @memberof lifelog.TimeRangeRequest
         * @static
         * @param {lifelog.ITimeRangeRequest} message TimeRangeRequest message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        TimeRangeRequest.encode = function encode(message, writer) {
            if (!writer)
                writer = $Writer.create();
            if (message.startTime != null && Object.hasOwnProperty.call(message, "startTime"))
                writer.uint32(/* id 1, wireType 2 =*/10).string(message.startTime);
            if (message.endTime != null && Object.hasOwnProperty.call(message, "endTime"))
                writer.uint32(/* id 2, wireType 2 =*/18).string(message.endTime);
            if (message.limit != null && Object.hasOwnProperty.call(message, "limit"))
                writer.uint32(/* id 3, wireType 0 =*/24).int32(message.limit);
            if (message.offset != null && Object.hasOwnProperty.call(message, "offset"))
                writer.uint32(/* id 4, wireType 0 =*/32).int32(message.offset);
            return writer;
        };

        /**
         * Encodes the specified TimeRangeRequest message, length delimited. Does not implicitly {@link lifelog.TimeRangeRequest.verify|verify} messages.
         * @function encodeDelimited
         * @memberof lifelog.TimeRangeRequest
         * @static
         * @param {lifelog.ITimeRangeRequest} message TimeRangeRequest message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        TimeRangeRequest.encodeDelimited = function encodeDelimited(message, writer) {
            return this.encode(message, writer).ldelim();
        };

        /**
         * Decodes a TimeRangeRequest message from the specified reader or buffer.
         * @function decode
         * @memberof lifelog.TimeRangeRequest
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @param {number} [length] Message length if known beforehand
         * @returns {lifelog.TimeRangeRequest} TimeRangeRequest
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        TimeRangeRequest.decode = function decode(reader, length, error) {
            if (!(reader instanceof $Reader))
                reader = $Reader.create(reader);
            let end = length === undefined ? reader.len : reader.pos + length, message = new $root.lifelog.TimeRangeRequest();
            while (reader.pos < end) {
                let tag = reader.uint32();
                if (tag === error)
                    break;
                switch (tag >>> 3) {
                case 1: {
                        message.startTime = reader.string();
                        break;
                    }
                case 2: {
                        message.endTime = reader.string();
                        break;
                    }
                case 3: {
                        message.limit = reader.int32();
                        break;
                    }
                case 4: {
                        message.offset = reader.int32();
                        break;
                    }
                default:
                    reader.skipType(tag & 7);
                    break;
                }
            }
            return message;
        };

        /**
         * Decodes a TimeRangeRequest message from the specified reader or buffer, length delimited.
         * @function decodeDelimited
         * @memberof lifelog.TimeRangeRequest
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @returns {lifelog.TimeRangeRequest} TimeRangeRequest
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        TimeRangeRequest.decodeDelimited = function decodeDelimited(reader) {
            if (!(reader instanceof $Reader))
                reader = new $Reader(reader);
            return this.decode(reader, reader.uint32());
        };

        /**
         * Verifies a TimeRangeRequest message.
         * @function verify
         * @memberof lifelog.TimeRangeRequest
         * @static
         * @param {Object.<string,*>} message Plain object to verify
         * @returns {string|null} `null` if valid, otherwise the reason why it is not
         */
        TimeRangeRequest.verify = function verify(message) {
            if (typeof message !== "object" || message === null)
                return "object expected";
            if (message.startTime != null && message.hasOwnProperty("startTime"))
                if (!$util.isString(message.startTime))
                    return "startTime: string expected";
            if (message.endTime != null && message.hasOwnProperty("endTime"))
                if (!$util.isString(message.endTime))
                    return "endTime: string expected";
            if (message.limit != null && message.hasOwnProperty("limit"))
                if (!$util.isInteger(message.limit))
                    return "limit: integer expected";
            if (message.offset != null && message.hasOwnProperty("offset"))
                if (!$util.isInteger(message.offset))
                    return "offset: integer expected";
            return null;
        };

        /**
         * Creates a TimeRangeRequest message from a plain object. Also converts values to their respective internal types.
         * @function fromObject
         * @memberof lifelog.TimeRangeRequest
         * @static
         * @param {Object.<string,*>} object Plain object
         * @returns {lifelog.TimeRangeRequest} TimeRangeRequest
         */
        TimeRangeRequest.fromObject = function fromObject(object) {
            if (object instanceof $root.lifelog.TimeRangeRequest)
                return object;
            let message = new $root.lifelog.TimeRangeRequest();
            if (object.startTime != null)
                message.startTime = String(object.startTime);
            if (object.endTime != null)
                message.endTime = String(object.endTime);
            if (object.limit != null)
                message.limit = object.limit | 0;
            if (object.offset != null)
                message.offset = object.offset | 0;
            return message;
        };

        /**
         * Creates a plain object from a TimeRangeRequest message. Also converts values to other types if specified.
         * @function toObject
         * @memberof lifelog.TimeRangeRequest
         * @static
         * @param {lifelog.TimeRangeRequest} message TimeRangeRequest
         * @param {$protobuf.IConversionOptions} [options] Conversion options
         * @returns {Object.<string,*>} Plain object
         */
        TimeRangeRequest.toObject = function toObject(message, options) {
            if (!options)
                options = {};
            let object = {};
            if (options.defaults) {
                object.startTime = "";
                object.endTime = "";
                object.limit = 0;
                object.offset = 0;
            }
            if (message.startTime != null && message.hasOwnProperty("startTime"))
                object.startTime = message.startTime;
            if (message.endTime != null && message.hasOwnProperty("endTime"))
                object.endTime = message.endTime;
            if (message.limit != null && message.hasOwnProperty("limit"))
                object.limit = message.limit;
            if (message.offset != null && message.hasOwnProperty("offset"))
                object.offset = message.offset;
            return object;
        };

        /**
         * Converts this TimeRangeRequest to JSON.
         * @function toJSON
         * @memberof lifelog.TimeRangeRequest
         * @instance
         * @returns {Object.<string,*>} JSON object
         */
        TimeRangeRequest.prototype.toJSON = function toJSON() {
            return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
        };

        /**
         * Gets the default type url for TimeRangeRequest
         * @function getTypeUrl
         * @memberof lifelog.TimeRangeRequest
         * @static
         * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns {string} The default type url
         */
        TimeRangeRequest.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
            if (typeUrlPrefix === undefined) {
                typeUrlPrefix = "type.googleapis.com";
            }
            return typeUrlPrefix + "/lifelog.TimeRangeRequest";
        };

        return TimeRangeRequest;
    })();

    lifelog.TimeStamped = (function() {

        /**
         * Properties of a TimeStamped.
         * @memberof lifelog
         * @interface ITimeStamped
         * @property {string|null} [timestamp] TimeStamped timestamp
         */

        /**
         * Constructs a new TimeStamped.
         * @memberof lifelog
         * @classdesc Represents a TimeStamped.
         * @implements ITimeStamped
         * @constructor
         * @param {lifelog.ITimeStamped=} [properties] Properties to set
         */
        function TimeStamped(properties) {
            if (properties)
                for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                    if (properties[keys[i]] != null)
                        this[keys[i]] = properties[keys[i]];
        }

        /**
         * TimeStamped timestamp.
         * @member {string} timestamp
         * @memberof lifelog.TimeStamped
         * @instance
         */
        TimeStamped.prototype.timestamp = "";

        /**
         * Creates a new TimeStamped instance using the specified properties.
         * @function create
         * @memberof lifelog.TimeStamped
         * @static
         * @param {lifelog.ITimeStamped=} [properties] Properties to set
         * @returns {lifelog.TimeStamped} TimeStamped instance
         */
        TimeStamped.create = function create(properties) {
            return new TimeStamped(properties);
        };

        /**
         * Encodes the specified TimeStamped message. Does not implicitly {@link lifelog.TimeStamped.verify|verify} messages.
         * @function encode
         * @memberof lifelog.TimeStamped
         * @static
         * @param {lifelog.ITimeStamped} message TimeStamped message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        TimeStamped.encode = function encode(message, writer) {
            if (!writer)
                writer = $Writer.create();
            if (message.timestamp != null && Object.hasOwnProperty.call(message, "timestamp"))
                writer.uint32(/* id 1, wireType 2 =*/10).string(message.timestamp);
            return writer;
        };

        /**
         * Encodes the specified TimeStamped message, length delimited. Does not implicitly {@link lifelog.TimeStamped.verify|verify} messages.
         * @function encodeDelimited
         * @memberof lifelog.TimeStamped
         * @static
         * @param {lifelog.ITimeStamped} message TimeStamped message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        TimeStamped.encodeDelimited = function encodeDelimited(message, writer) {
            return this.encode(message, writer).ldelim();
        };

        /**
         * Decodes a TimeStamped message from the specified reader or buffer.
         * @function decode
         * @memberof lifelog.TimeStamped
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @param {number} [length] Message length if known beforehand
         * @returns {lifelog.TimeStamped} TimeStamped
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        TimeStamped.decode = function decode(reader, length, error) {
            if (!(reader instanceof $Reader))
                reader = $Reader.create(reader);
            let end = length === undefined ? reader.len : reader.pos + length, message = new $root.lifelog.TimeStamped();
            while (reader.pos < end) {
                let tag = reader.uint32();
                if (tag === error)
                    break;
                switch (tag >>> 3) {
                case 1: {
                        message.timestamp = reader.string();
                        break;
                    }
                default:
                    reader.skipType(tag & 7);
                    break;
                }
            }
            return message;
        };

        /**
         * Decodes a TimeStamped message from the specified reader or buffer, length delimited.
         * @function decodeDelimited
         * @memberof lifelog.TimeStamped
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @returns {lifelog.TimeStamped} TimeStamped
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        TimeStamped.decodeDelimited = function decodeDelimited(reader) {
            if (!(reader instanceof $Reader))
                reader = new $Reader(reader);
            return this.decode(reader, reader.uint32());
        };

        /**
         * Verifies a TimeStamped message.
         * @function verify
         * @memberof lifelog.TimeStamped
         * @static
         * @param {Object.<string,*>} message Plain object to verify
         * @returns {string|null} `null` if valid, otherwise the reason why it is not
         */
        TimeStamped.verify = function verify(message) {
            if (typeof message !== "object" || message === null)
                return "object expected";
            if (message.timestamp != null && message.hasOwnProperty("timestamp"))
                if (!$util.isString(message.timestamp))
                    return "timestamp: string expected";
            return null;
        };

        /**
         * Creates a TimeStamped message from a plain object. Also converts values to their respective internal types.
         * @function fromObject
         * @memberof lifelog.TimeStamped
         * @static
         * @param {Object.<string,*>} object Plain object
         * @returns {lifelog.TimeStamped} TimeStamped
         */
        TimeStamped.fromObject = function fromObject(object) {
            if (object instanceof $root.lifelog.TimeStamped)
                return object;
            let message = new $root.lifelog.TimeStamped();
            if (object.timestamp != null)
                message.timestamp = String(object.timestamp);
            return message;
        };

        /**
         * Creates a plain object from a TimeStamped message. Also converts values to other types if specified.
         * @function toObject
         * @memberof lifelog.TimeStamped
         * @static
         * @param {lifelog.TimeStamped} message TimeStamped
         * @param {$protobuf.IConversionOptions} [options] Conversion options
         * @returns {Object.<string,*>} Plain object
         */
        TimeStamped.toObject = function toObject(message, options) {
            if (!options)
                options = {};
            let object = {};
            if (options.defaults)
                object.timestamp = "";
            if (message.timestamp != null && message.hasOwnProperty("timestamp"))
                object.timestamp = message.timestamp;
            return object;
        };

        /**
         * Converts this TimeStamped to JSON.
         * @function toJSON
         * @memberof lifelog.TimeStamped
         * @instance
         * @returns {Object.<string,*>} JSON object
         */
        TimeStamped.prototype.toJSON = function toJSON() {
            return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
        };

        /**
         * Gets the default type url for TimeStamped
         * @function getTypeUrl
         * @memberof lifelog.TimeStamped
         * @static
         * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns {string} The default type url
         */
        TimeStamped.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
            if (typeUrlPrefix === undefined) {
                typeUrlPrefix = "type.googleapis.com";
            }
            return typeUrlPrefix + "/lifelog.TimeStamped";
        };

        return TimeStamped;
    })();

    lifelog.SearchRequest = (function() {

        /**
         * Properties of a SearchRequest.
         * @memberof lifelog
         * @interface ISearchRequest
         * @property {string|null} [query] SearchRequest query
         * @property {Array.<string>|null} [dataSources] SearchRequest dataSources
         * @property {lifelog.ITimeRangeRequest|null} [timeRange] SearchRequest timeRange
         * @property {boolean|null} [useLlm] SearchRequest useLlm
         */

        /**
         * Constructs a new SearchRequest.
         * @memberof lifelog
         * @classdesc Represents a SearchRequest.
         * @implements ISearchRequest
         * @constructor
         * @param {lifelog.ISearchRequest=} [properties] Properties to set
         */
        function SearchRequest(properties) {
            this.dataSources = [];
            if (properties)
                for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                    if (properties[keys[i]] != null)
                        this[keys[i]] = properties[keys[i]];
        }

        /**
         * SearchRequest query.
         * @member {string} query
         * @memberof lifelog.SearchRequest
         * @instance
         */
        SearchRequest.prototype.query = "";

        /**
         * SearchRequest dataSources.
         * @member {Array.<string>} dataSources
         * @memberof lifelog.SearchRequest
         * @instance
         */
        SearchRequest.prototype.dataSources = $util.emptyArray;

        /**
         * SearchRequest timeRange.
         * @member {lifelog.ITimeRangeRequest|null|undefined} timeRange
         * @memberof lifelog.SearchRequest
         * @instance
         */
        SearchRequest.prototype.timeRange = null;

        /**
         * SearchRequest useLlm.
         * @member {boolean} useLlm
         * @memberof lifelog.SearchRequest
         * @instance
         */
        SearchRequest.prototype.useLlm = false;

        /**
         * Creates a new SearchRequest instance using the specified properties.
         * @function create
         * @memberof lifelog.SearchRequest
         * @static
         * @param {lifelog.ISearchRequest=} [properties] Properties to set
         * @returns {lifelog.SearchRequest} SearchRequest instance
         */
        SearchRequest.create = function create(properties) {
            return new SearchRequest(properties);
        };

        /**
         * Encodes the specified SearchRequest message. Does not implicitly {@link lifelog.SearchRequest.verify|verify} messages.
         * @function encode
         * @memberof lifelog.SearchRequest
         * @static
         * @param {lifelog.ISearchRequest} message SearchRequest message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        SearchRequest.encode = function encode(message, writer) {
            if (!writer)
                writer = $Writer.create();
            if (message.query != null && Object.hasOwnProperty.call(message, "query"))
                writer.uint32(/* id 1, wireType 2 =*/10).string(message.query);
            if (message.dataSources != null && message.dataSources.length)
                for (let i = 0; i < message.dataSources.length; ++i)
                    writer.uint32(/* id 2, wireType 2 =*/18).string(message.dataSources[i]);
            if (message.timeRange != null && Object.hasOwnProperty.call(message, "timeRange"))
                $root.lifelog.TimeRangeRequest.encode(message.timeRange, writer.uint32(/* id 3, wireType 2 =*/26).fork()).ldelim();
            if (message.useLlm != null && Object.hasOwnProperty.call(message, "useLlm"))
                writer.uint32(/* id 4, wireType 0 =*/32).bool(message.useLlm);
            return writer;
        };

        /**
         * Encodes the specified SearchRequest message, length delimited. Does not implicitly {@link lifelog.SearchRequest.verify|verify} messages.
         * @function encodeDelimited
         * @memberof lifelog.SearchRequest
         * @static
         * @param {lifelog.ISearchRequest} message SearchRequest message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        SearchRequest.encodeDelimited = function encodeDelimited(message, writer) {
            return this.encode(message, writer).ldelim();
        };

        /**
         * Decodes a SearchRequest message from the specified reader or buffer.
         * @function decode
         * @memberof lifelog.SearchRequest
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @param {number} [length] Message length if known beforehand
         * @returns {lifelog.SearchRequest} SearchRequest
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        SearchRequest.decode = function decode(reader, length, error) {
            if (!(reader instanceof $Reader))
                reader = $Reader.create(reader);
            let end = length === undefined ? reader.len : reader.pos + length, message = new $root.lifelog.SearchRequest();
            while (reader.pos < end) {
                let tag = reader.uint32();
                if (tag === error)
                    break;
                switch (tag >>> 3) {
                case 1: {
                        message.query = reader.string();
                        break;
                    }
                case 2: {
                        if (!(message.dataSources && message.dataSources.length))
                            message.dataSources = [];
                        message.dataSources.push(reader.string());
                        break;
                    }
                case 3: {
                        message.timeRange = $root.lifelog.TimeRangeRequest.decode(reader, reader.uint32());
                        break;
                    }
                case 4: {
                        message.useLlm = reader.bool();
                        break;
                    }
                default:
                    reader.skipType(tag & 7);
                    break;
                }
            }
            return message;
        };

        /**
         * Decodes a SearchRequest message from the specified reader or buffer, length delimited.
         * @function decodeDelimited
         * @memberof lifelog.SearchRequest
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @returns {lifelog.SearchRequest} SearchRequest
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        SearchRequest.decodeDelimited = function decodeDelimited(reader) {
            if (!(reader instanceof $Reader))
                reader = new $Reader(reader);
            return this.decode(reader, reader.uint32());
        };

        /**
         * Verifies a SearchRequest message.
         * @function verify
         * @memberof lifelog.SearchRequest
         * @static
         * @param {Object.<string,*>} message Plain object to verify
         * @returns {string|null} `null` if valid, otherwise the reason why it is not
         */
        SearchRequest.verify = function verify(message) {
            if (typeof message !== "object" || message === null)
                return "object expected";
            if (message.query != null && message.hasOwnProperty("query"))
                if (!$util.isString(message.query))
                    return "query: string expected";
            if (message.dataSources != null && message.hasOwnProperty("dataSources")) {
                if (!Array.isArray(message.dataSources))
                    return "dataSources: array expected";
                for (let i = 0; i < message.dataSources.length; ++i)
                    if (!$util.isString(message.dataSources[i]))
                        return "dataSources: string[] expected";
            }
            if (message.timeRange != null && message.hasOwnProperty("timeRange")) {
                let error = $root.lifelog.TimeRangeRequest.verify(message.timeRange);
                if (error)
                    return "timeRange." + error;
            }
            if (message.useLlm != null && message.hasOwnProperty("useLlm"))
                if (typeof message.useLlm !== "boolean")
                    return "useLlm: boolean expected";
            return null;
        };

        /**
         * Creates a SearchRequest message from a plain object. Also converts values to their respective internal types.
         * @function fromObject
         * @memberof lifelog.SearchRequest
         * @static
         * @param {Object.<string,*>} object Plain object
         * @returns {lifelog.SearchRequest} SearchRequest
         */
        SearchRequest.fromObject = function fromObject(object) {
            if (object instanceof $root.lifelog.SearchRequest)
                return object;
            let message = new $root.lifelog.SearchRequest();
            if (object.query != null)
                message.query = String(object.query);
            if (object.dataSources) {
                if (!Array.isArray(object.dataSources))
                    throw TypeError(".lifelog.SearchRequest.dataSources: array expected");
                message.dataSources = [];
                for (let i = 0; i < object.dataSources.length; ++i)
                    message.dataSources[i] = String(object.dataSources[i]);
            }
            if (object.timeRange != null) {
                if (typeof object.timeRange !== "object")
                    throw TypeError(".lifelog.SearchRequest.timeRange: object expected");
                message.timeRange = $root.lifelog.TimeRangeRequest.fromObject(object.timeRange);
            }
            if (object.useLlm != null)
                message.useLlm = Boolean(object.useLlm);
            return message;
        };

        /**
         * Creates a plain object from a SearchRequest message. Also converts values to other types if specified.
         * @function toObject
         * @memberof lifelog.SearchRequest
         * @static
         * @param {lifelog.SearchRequest} message SearchRequest
         * @param {$protobuf.IConversionOptions} [options] Conversion options
         * @returns {Object.<string,*>} Plain object
         */
        SearchRequest.toObject = function toObject(message, options) {
            if (!options)
                options = {};
            let object = {};
            if (options.arrays || options.defaults)
                object.dataSources = [];
            if (options.defaults) {
                object.query = "";
                object.timeRange = null;
                object.useLlm = false;
            }
            if (message.query != null && message.hasOwnProperty("query"))
                object.query = message.query;
            if (message.dataSources && message.dataSources.length) {
                object.dataSources = [];
                for (let j = 0; j < message.dataSources.length; ++j)
                    object.dataSources[j] = message.dataSources[j];
            }
            if (message.timeRange != null && message.hasOwnProperty("timeRange"))
                object.timeRange = $root.lifelog.TimeRangeRequest.toObject(message.timeRange, options);
            if (message.useLlm != null && message.hasOwnProperty("useLlm"))
                object.useLlm = message.useLlm;
            return object;
        };

        /**
         * Converts this SearchRequest to JSON.
         * @function toJSON
         * @memberof lifelog.SearchRequest
         * @instance
         * @returns {Object.<string,*>} JSON object
         */
        SearchRequest.prototype.toJSON = function toJSON() {
            return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
        };

        /**
         * Gets the default type url for SearchRequest
         * @function getTypeUrl
         * @memberof lifelog.SearchRequest
         * @static
         * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns {string} The default type url
         */
        SearchRequest.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
            if (typeUrlPrefix === undefined) {
                typeUrlPrefix = "type.googleapis.com";
            }
            return typeUrlPrefix + "/lifelog.SearchRequest";
        };

        return SearchRequest;
    })();

    lifelog.SearchResult = (function() {

        /**
         * Properties of a SearchResult.
         * @memberof lifelog
         * @interface ISearchResult
         * @property {string|null} [type] SearchResult type
         * @property {string|null} [timestamp] SearchResult timestamp
         * @property {string|null} [sourceId] SearchResult sourceId
         * @property {Object.<string,string>|null} [metadata] SearchResult metadata
         * @property {Uint8Array|null} [binaryData] SearchResult binaryData
         * @property {string|null} [textData] SearchResult textData
         * @property {number|null} [relevanceScore] SearchResult relevanceScore
         */

        /**
         * Constructs a new SearchResult.
         * @memberof lifelog
         * @classdesc Represents a SearchResult.
         * @implements ISearchResult
         * @constructor
         * @param {lifelog.ISearchResult=} [properties] Properties to set
         */
        function SearchResult(properties) {
            this.metadata = {};
            if (properties)
                for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                    if (properties[keys[i]] != null)
                        this[keys[i]] = properties[keys[i]];
        }

        /**
         * SearchResult type.
         * @member {string} type
         * @memberof lifelog.SearchResult
         * @instance
         */
        SearchResult.prototype.type = "";

        /**
         * SearchResult timestamp.
         * @member {string} timestamp
         * @memberof lifelog.SearchResult
         * @instance
         */
        SearchResult.prototype.timestamp = "";

        /**
         * SearchResult sourceId.
         * @member {string} sourceId
         * @memberof lifelog.SearchResult
         * @instance
         */
        SearchResult.prototype.sourceId = "";

        /**
         * SearchResult metadata.
         * @member {Object.<string,string>} metadata
         * @memberof lifelog.SearchResult
         * @instance
         */
        SearchResult.prototype.metadata = $util.emptyObject;

        /**
         * SearchResult binaryData.
         * @member {Uint8Array|null|undefined} binaryData
         * @memberof lifelog.SearchResult
         * @instance
         */
        SearchResult.prototype.binaryData = null;

        /**
         * SearchResult textData.
         * @member {string|null|undefined} textData
         * @memberof lifelog.SearchResult
         * @instance
         */
        SearchResult.prototype.textData = null;

        /**
         * SearchResult relevanceScore.
         * @member {number} relevanceScore
         * @memberof lifelog.SearchResult
         * @instance
         */
        SearchResult.prototype.relevanceScore = 0;

        // OneOf field names bound to virtual getters and setters
        let $oneOfFields;

        /**
         * SearchResult data.
         * @member {"binaryData"|"textData"|undefined} data
         * @memberof lifelog.SearchResult
         * @instance
         */
        Object.defineProperty(SearchResult.prototype, "data", {
            get: $util.oneOfGetter($oneOfFields = ["binaryData", "textData"]),
            set: $util.oneOfSetter($oneOfFields)
        });

        /**
         * Creates a new SearchResult instance using the specified properties.
         * @function create
         * @memberof lifelog.SearchResult
         * @static
         * @param {lifelog.ISearchResult=} [properties] Properties to set
         * @returns {lifelog.SearchResult} SearchResult instance
         */
        SearchResult.create = function create(properties) {
            return new SearchResult(properties);
        };

        /**
         * Encodes the specified SearchResult message. Does not implicitly {@link lifelog.SearchResult.verify|verify} messages.
         * @function encode
         * @memberof lifelog.SearchResult
         * @static
         * @param {lifelog.ISearchResult} message SearchResult message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        SearchResult.encode = function encode(message, writer) {
            if (!writer)
                writer = $Writer.create();
            if (message.type != null && Object.hasOwnProperty.call(message, "type"))
                writer.uint32(/* id 1, wireType 2 =*/10).string(message.type);
            if (message.timestamp != null && Object.hasOwnProperty.call(message, "timestamp"))
                writer.uint32(/* id 2, wireType 2 =*/18).string(message.timestamp);
            if (message.sourceId != null && Object.hasOwnProperty.call(message, "sourceId"))
                writer.uint32(/* id 3, wireType 2 =*/26).string(message.sourceId);
            if (message.metadata != null && Object.hasOwnProperty.call(message, "metadata"))
                for (let keys = Object.keys(message.metadata), i = 0; i < keys.length; ++i)
                    writer.uint32(/* id 4, wireType 2 =*/34).fork().uint32(/* id 1, wireType 2 =*/10).string(keys[i]).uint32(/* id 2, wireType 2 =*/18).string(message.metadata[keys[i]]).ldelim();
            if (message.binaryData != null && Object.hasOwnProperty.call(message, "binaryData"))
                writer.uint32(/* id 5, wireType 2 =*/42).bytes(message.binaryData);
            if (message.textData != null && Object.hasOwnProperty.call(message, "textData"))
                writer.uint32(/* id 6, wireType 2 =*/50).string(message.textData);
            if (message.relevanceScore != null && Object.hasOwnProperty.call(message, "relevanceScore"))
                writer.uint32(/* id 7, wireType 5 =*/61).float(message.relevanceScore);
            return writer;
        };

        /**
         * Encodes the specified SearchResult message, length delimited. Does not implicitly {@link lifelog.SearchResult.verify|verify} messages.
         * @function encodeDelimited
         * @memberof lifelog.SearchResult
         * @static
         * @param {lifelog.ISearchResult} message SearchResult message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        SearchResult.encodeDelimited = function encodeDelimited(message, writer) {
            return this.encode(message, writer).ldelim();
        };

        /**
         * Decodes a SearchResult message from the specified reader or buffer.
         * @function decode
         * @memberof lifelog.SearchResult
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @param {number} [length] Message length if known beforehand
         * @returns {lifelog.SearchResult} SearchResult
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        SearchResult.decode = function decode(reader, length, error) {
            if (!(reader instanceof $Reader))
                reader = $Reader.create(reader);
            let end = length === undefined ? reader.len : reader.pos + length, message = new $root.lifelog.SearchResult(), key, value;
            while (reader.pos < end) {
                let tag = reader.uint32();
                if (tag === error)
                    break;
                switch (tag >>> 3) {
                case 1: {
                        message.type = reader.string();
                        break;
                    }
                case 2: {
                        message.timestamp = reader.string();
                        break;
                    }
                case 3: {
                        message.sourceId = reader.string();
                        break;
                    }
                case 4: {
                        if (message.metadata === $util.emptyObject)
                            message.metadata = {};
                        let end2 = reader.uint32() + reader.pos;
                        key = "";
                        value = "";
                        while (reader.pos < end2) {
                            let tag2 = reader.uint32();
                            switch (tag2 >>> 3) {
                            case 1:
                                key = reader.string();
                                break;
                            case 2:
                                value = reader.string();
                                break;
                            default:
                                reader.skipType(tag2 & 7);
                                break;
                            }
                        }
                        message.metadata[key] = value;
                        break;
                    }
                case 5: {
                        message.binaryData = reader.bytes();
                        break;
                    }
                case 6: {
                        message.textData = reader.string();
                        break;
                    }
                case 7: {
                        message.relevanceScore = reader.float();
                        break;
                    }
                default:
                    reader.skipType(tag & 7);
                    break;
                }
            }
            return message;
        };

        /**
         * Decodes a SearchResult message from the specified reader or buffer, length delimited.
         * @function decodeDelimited
         * @memberof lifelog.SearchResult
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @returns {lifelog.SearchResult} SearchResult
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        SearchResult.decodeDelimited = function decodeDelimited(reader) {
            if (!(reader instanceof $Reader))
                reader = new $Reader(reader);
            return this.decode(reader, reader.uint32());
        };

        /**
         * Verifies a SearchResult message.
         * @function verify
         * @memberof lifelog.SearchResult
         * @static
         * @param {Object.<string,*>} message Plain object to verify
         * @returns {string|null} `null` if valid, otherwise the reason why it is not
         */
        SearchResult.verify = function verify(message) {
            if (typeof message !== "object" || message === null)
                return "object expected";
            let properties = {};
            if (message.type != null && message.hasOwnProperty("type"))
                if (!$util.isString(message.type))
                    return "type: string expected";
            if (message.timestamp != null && message.hasOwnProperty("timestamp"))
                if (!$util.isString(message.timestamp))
                    return "timestamp: string expected";
            if (message.sourceId != null && message.hasOwnProperty("sourceId"))
                if (!$util.isString(message.sourceId))
                    return "sourceId: string expected";
            if (message.metadata != null && message.hasOwnProperty("metadata")) {
                if (!$util.isObject(message.metadata))
                    return "metadata: object expected";
                let key = Object.keys(message.metadata);
                for (let i = 0; i < key.length; ++i)
                    if (!$util.isString(message.metadata[key[i]]))
                        return "metadata: string{k:string} expected";
            }
            if (message.binaryData != null && message.hasOwnProperty("binaryData")) {
                properties.data = 1;
                if (!(message.binaryData && typeof message.binaryData.length === "number" || $util.isString(message.binaryData)))
                    return "binaryData: buffer expected";
            }
            if (message.textData != null && message.hasOwnProperty("textData")) {
                if (properties.data === 1)
                    return "data: multiple values";
                properties.data = 1;
                if (!$util.isString(message.textData))
                    return "textData: string expected";
            }
            if (message.relevanceScore != null && message.hasOwnProperty("relevanceScore"))
                if (typeof message.relevanceScore !== "number")
                    return "relevanceScore: number expected";
            return null;
        };

        /**
         * Creates a SearchResult message from a plain object. Also converts values to their respective internal types.
         * @function fromObject
         * @memberof lifelog.SearchResult
         * @static
         * @param {Object.<string,*>} object Plain object
         * @returns {lifelog.SearchResult} SearchResult
         */
        SearchResult.fromObject = function fromObject(object) {
            if (object instanceof $root.lifelog.SearchResult)
                return object;
            let message = new $root.lifelog.SearchResult();
            if (object.type != null)
                message.type = String(object.type);
            if (object.timestamp != null)
                message.timestamp = String(object.timestamp);
            if (object.sourceId != null)
                message.sourceId = String(object.sourceId);
            if (object.metadata) {
                if (typeof object.metadata !== "object")
                    throw TypeError(".lifelog.SearchResult.metadata: object expected");
                message.metadata = {};
                for (let keys = Object.keys(object.metadata), i = 0; i < keys.length; ++i)
                    message.metadata[keys[i]] = String(object.metadata[keys[i]]);
            }
            if (object.binaryData != null)
                if (typeof object.binaryData === "string")
                    $util.base64.decode(object.binaryData, message.binaryData = $util.newBuffer($util.base64.length(object.binaryData)), 0);
                else if (object.binaryData.length >= 0)
                    message.binaryData = object.binaryData;
            if (object.textData != null)
                message.textData = String(object.textData);
            if (object.relevanceScore != null)
                message.relevanceScore = Number(object.relevanceScore);
            return message;
        };

        /**
         * Creates a plain object from a SearchResult message. Also converts values to other types if specified.
         * @function toObject
         * @memberof lifelog.SearchResult
         * @static
         * @param {lifelog.SearchResult} message SearchResult
         * @param {$protobuf.IConversionOptions} [options] Conversion options
         * @returns {Object.<string,*>} Plain object
         */
        SearchResult.toObject = function toObject(message, options) {
            if (!options)
                options = {};
            let object = {};
            if (options.objects || options.defaults)
                object.metadata = {};
            if (options.defaults) {
                object.type = "";
                object.timestamp = "";
                object.sourceId = "";
                object.relevanceScore = 0;
            }
            if (message.type != null && message.hasOwnProperty("type"))
                object.type = message.type;
            if (message.timestamp != null && message.hasOwnProperty("timestamp"))
                object.timestamp = message.timestamp;
            if (message.sourceId != null && message.hasOwnProperty("sourceId"))
                object.sourceId = message.sourceId;
            let keys2;
            if (message.metadata && (keys2 = Object.keys(message.metadata)).length) {
                object.metadata = {};
                for (let j = 0; j < keys2.length; ++j)
                    object.metadata[keys2[j]] = message.metadata[keys2[j]];
            }
            if (message.binaryData != null && message.hasOwnProperty("binaryData")) {
                object.binaryData = options.bytes === String ? $util.base64.encode(message.binaryData, 0, message.binaryData.length) : options.bytes === Array ? Array.prototype.slice.call(message.binaryData) : message.binaryData;
                if (options.oneofs)
                    object.data = "binaryData";
            }
            if (message.textData != null && message.hasOwnProperty("textData")) {
                object.textData = message.textData;
                if (options.oneofs)
                    object.data = "textData";
            }
            if (message.relevanceScore != null && message.hasOwnProperty("relevanceScore"))
                object.relevanceScore = options.json && !isFinite(message.relevanceScore) ? String(message.relevanceScore) : message.relevanceScore;
            return object;
        };

        /**
         * Converts this SearchResult to JSON.
         * @function toJSON
         * @memberof lifelog.SearchResult
         * @instance
         * @returns {Object.<string,*>} JSON object
         */
        SearchResult.prototype.toJSON = function toJSON() {
            return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
        };

        /**
         * Gets the default type url for SearchResult
         * @function getTypeUrl
         * @memberof lifelog.SearchResult
         * @static
         * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns {string} The default type url
         */
        SearchResult.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
            if (typeUrlPrefix === undefined) {
                typeUrlPrefix = "type.googleapis.com";
            }
            return typeUrlPrefix + "/lifelog.SearchResult";
        };

        return SearchResult;
    })();

    lifelog.SearchResponse = (function() {

        /**
         * Properties of a SearchResponse.
         * @memberof lifelog
         * @interface ISearchResponse
         * @property {Array.<lifelog.ISearchResult>|null} [results] SearchResponse results
         * @property {number|null} [totalResults] SearchResponse totalResults
         * @property {string|null} [searchId] SearchResponse searchId
         */

        /**
         * Constructs a new SearchResponse.
         * @memberof lifelog
         * @classdesc Represents a SearchResponse.
         * @implements ISearchResponse
         * @constructor
         * @param {lifelog.ISearchResponse=} [properties] Properties to set
         */
        function SearchResponse(properties) {
            this.results = [];
            if (properties)
                for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                    if (properties[keys[i]] != null)
                        this[keys[i]] = properties[keys[i]];
        }

        /**
         * SearchResponse results.
         * @member {Array.<lifelog.ISearchResult>} results
         * @memberof lifelog.SearchResponse
         * @instance
         */
        SearchResponse.prototype.results = $util.emptyArray;

        /**
         * SearchResponse totalResults.
         * @member {number} totalResults
         * @memberof lifelog.SearchResponse
         * @instance
         */
        SearchResponse.prototype.totalResults = 0;

        /**
         * SearchResponse searchId.
         * @member {string} searchId
         * @memberof lifelog.SearchResponse
         * @instance
         */
        SearchResponse.prototype.searchId = "";

        /**
         * Creates a new SearchResponse instance using the specified properties.
         * @function create
         * @memberof lifelog.SearchResponse
         * @static
         * @param {lifelog.ISearchResponse=} [properties] Properties to set
         * @returns {lifelog.SearchResponse} SearchResponse instance
         */
        SearchResponse.create = function create(properties) {
            return new SearchResponse(properties);
        };

        /**
         * Encodes the specified SearchResponse message. Does not implicitly {@link lifelog.SearchResponse.verify|verify} messages.
         * @function encode
         * @memberof lifelog.SearchResponse
         * @static
         * @param {lifelog.ISearchResponse} message SearchResponse message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        SearchResponse.encode = function encode(message, writer) {
            if (!writer)
                writer = $Writer.create();
            if (message.results != null && message.results.length)
                for (let i = 0; i < message.results.length; ++i)
                    $root.lifelog.SearchResult.encode(message.results[i], writer.uint32(/* id 1, wireType 2 =*/10).fork()).ldelim();
            if (message.totalResults != null && Object.hasOwnProperty.call(message, "totalResults"))
                writer.uint32(/* id 2, wireType 0 =*/16).int32(message.totalResults);
            if (message.searchId != null && Object.hasOwnProperty.call(message, "searchId"))
                writer.uint32(/* id 3, wireType 2 =*/26).string(message.searchId);
            return writer;
        };

        /**
         * Encodes the specified SearchResponse message, length delimited. Does not implicitly {@link lifelog.SearchResponse.verify|verify} messages.
         * @function encodeDelimited
         * @memberof lifelog.SearchResponse
         * @static
         * @param {lifelog.ISearchResponse} message SearchResponse message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        SearchResponse.encodeDelimited = function encodeDelimited(message, writer) {
            return this.encode(message, writer).ldelim();
        };

        /**
         * Decodes a SearchResponse message from the specified reader or buffer.
         * @function decode
         * @memberof lifelog.SearchResponse
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @param {number} [length] Message length if known beforehand
         * @returns {lifelog.SearchResponse} SearchResponse
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        SearchResponse.decode = function decode(reader, length, error) {
            if (!(reader instanceof $Reader))
                reader = $Reader.create(reader);
            let end = length === undefined ? reader.len : reader.pos + length, message = new $root.lifelog.SearchResponse();
            while (reader.pos < end) {
                let tag = reader.uint32();
                if (tag === error)
                    break;
                switch (tag >>> 3) {
                case 1: {
                        if (!(message.results && message.results.length))
                            message.results = [];
                        message.results.push($root.lifelog.SearchResult.decode(reader, reader.uint32()));
                        break;
                    }
                case 2: {
                        message.totalResults = reader.int32();
                        break;
                    }
                case 3: {
                        message.searchId = reader.string();
                        break;
                    }
                default:
                    reader.skipType(tag & 7);
                    break;
                }
            }
            return message;
        };

        /**
         * Decodes a SearchResponse message from the specified reader or buffer, length delimited.
         * @function decodeDelimited
         * @memberof lifelog.SearchResponse
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @returns {lifelog.SearchResponse} SearchResponse
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        SearchResponse.decodeDelimited = function decodeDelimited(reader) {
            if (!(reader instanceof $Reader))
                reader = new $Reader(reader);
            return this.decode(reader, reader.uint32());
        };

        /**
         * Verifies a SearchResponse message.
         * @function verify
         * @memberof lifelog.SearchResponse
         * @static
         * @param {Object.<string,*>} message Plain object to verify
         * @returns {string|null} `null` if valid, otherwise the reason why it is not
         */
        SearchResponse.verify = function verify(message) {
            if (typeof message !== "object" || message === null)
                return "object expected";
            if (message.results != null && message.hasOwnProperty("results")) {
                if (!Array.isArray(message.results))
                    return "results: array expected";
                for (let i = 0; i < message.results.length; ++i) {
                    let error = $root.lifelog.SearchResult.verify(message.results[i]);
                    if (error)
                        return "results." + error;
                }
            }
            if (message.totalResults != null && message.hasOwnProperty("totalResults"))
                if (!$util.isInteger(message.totalResults))
                    return "totalResults: integer expected";
            if (message.searchId != null && message.hasOwnProperty("searchId"))
                if (!$util.isString(message.searchId))
                    return "searchId: string expected";
            return null;
        };

        /**
         * Creates a SearchResponse message from a plain object. Also converts values to their respective internal types.
         * @function fromObject
         * @memberof lifelog.SearchResponse
         * @static
         * @param {Object.<string,*>} object Plain object
         * @returns {lifelog.SearchResponse} SearchResponse
         */
        SearchResponse.fromObject = function fromObject(object) {
            if (object instanceof $root.lifelog.SearchResponse)
                return object;
            let message = new $root.lifelog.SearchResponse();
            if (object.results) {
                if (!Array.isArray(object.results))
                    throw TypeError(".lifelog.SearchResponse.results: array expected");
                message.results = [];
                for (let i = 0; i < object.results.length; ++i) {
                    if (typeof object.results[i] !== "object")
                        throw TypeError(".lifelog.SearchResponse.results: object expected");
                    message.results[i] = $root.lifelog.SearchResult.fromObject(object.results[i]);
                }
            }
            if (object.totalResults != null)
                message.totalResults = object.totalResults | 0;
            if (object.searchId != null)
                message.searchId = String(object.searchId);
            return message;
        };

        /**
         * Creates a plain object from a SearchResponse message. Also converts values to other types if specified.
         * @function toObject
         * @memberof lifelog.SearchResponse
         * @static
         * @param {lifelog.SearchResponse} message SearchResponse
         * @param {$protobuf.IConversionOptions} [options] Conversion options
         * @returns {Object.<string,*>} Plain object
         */
        SearchResponse.toObject = function toObject(message, options) {
            if (!options)
                options = {};
            let object = {};
            if (options.arrays || options.defaults)
                object.results = [];
            if (options.defaults) {
                object.totalResults = 0;
                object.searchId = "";
            }
            if (message.results && message.results.length) {
                object.results = [];
                for (let j = 0; j < message.results.length; ++j)
                    object.results[j] = $root.lifelog.SearchResult.toObject(message.results[j], options);
            }
            if (message.totalResults != null && message.hasOwnProperty("totalResults"))
                object.totalResults = message.totalResults;
            if (message.searchId != null && message.hasOwnProperty("searchId"))
                object.searchId = message.searchId;
            return object;
        };

        /**
         * Converts this SearchResponse to JSON.
         * @function toJSON
         * @memberof lifelog.SearchResponse
         * @instance
         * @returns {Object.<string,*>} JSON object
         */
        SearchResponse.prototype.toJSON = function toJSON() {
            return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
        };

        /**
         * Gets the default type url for SearchResponse
         * @function getTypeUrl
         * @memberof lifelog.SearchResponse
         * @static
         * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns {string} The default type url
         */
        SearchResponse.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
            if (typeUrlPrefix === undefined) {
                typeUrlPrefix = "type.googleapis.com";
            }
            return typeUrlPrefix + "/lifelog.SearchResponse";
        };

        return SearchResponse;
    })();

    lifelog.ScreenshotData = (function() {

        /**
         * Properties of a ScreenshotData.
         * @memberof lifelog
         * @interface IScreenshotData
         * @property {string|null} [id] ScreenshotData id
         * @property {string|null} [timestamp] ScreenshotData timestamp
         * @property {Uint8Array|null} [imageData] ScreenshotData imageData
         * @property {string|null} [mimeType] ScreenshotData mimeType
         * @property {Object.<string,string>|null} [metadata] ScreenshotData metadata
         */

        /**
         * Constructs a new ScreenshotData.
         * @memberof lifelog
         * @classdesc Represents a ScreenshotData.
         * @implements IScreenshotData
         * @constructor
         * @param {lifelog.IScreenshotData=} [properties] Properties to set
         */
        function ScreenshotData(properties) {
            this.metadata = {};
            if (properties)
                for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                    if (properties[keys[i]] != null)
                        this[keys[i]] = properties[keys[i]];
        }

        /**
         * ScreenshotData id.
         * @member {string} id
         * @memberof lifelog.ScreenshotData
         * @instance
         */
        ScreenshotData.prototype.id = "";

        /**
         * ScreenshotData timestamp.
         * @member {string} timestamp
         * @memberof lifelog.ScreenshotData
         * @instance
         */
        ScreenshotData.prototype.timestamp = "";

        /**
         * ScreenshotData imageData.
         * @member {Uint8Array} imageData
         * @memberof lifelog.ScreenshotData
         * @instance
         */
        ScreenshotData.prototype.imageData = $util.newBuffer([]);

        /**
         * ScreenshotData mimeType.
         * @member {string} mimeType
         * @memberof lifelog.ScreenshotData
         * @instance
         */
        ScreenshotData.prototype.mimeType = "";

        /**
         * ScreenshotData metadata.
         * @member {Object.<string,string>} metadata
         * @memberof lifelog.ScreenshotData
         * @instance
         */
        ScreenshotData.prototype.metadata = $util.emptyObject;

        /**
         * Creates a new ScreenshotData instance using the specified properties.
         * @function create
         * @memberof lifelog.ScreenshotData
         * @static
         * @param {lifelog.IScreenshotData=} [properties] Properties to set
         * @returns {lifelog.ScreenshotData} ScreenshotData instance
         */
        ScreenshotData.create = function create(properties) {
            return new ScreenshotData(properties);
        };

        /**
         * Encodes the specified ScreenshotData message. Does not implicitly {@link lifelog.ScreenshotData.verify|verify} messages.
         * @function encode
         * @memberof lifelog.ScreenshotData
         * @static
         * @param {lifelog.IScreenshotData} message ScreenshotData message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        ScreenshotData.encode = function encode(message, writer) {
            if (!writer)
                writer = $Writer.create();
            if (message.id != null && Object.hasOwnProperty.call(message, "id"))
                writer.uint32(/* id 1, wireType 2 =*/10).string(message.id);
            if (message.timestamp != null && Object.hasOwnProperty.call(message, "timestamp"))
                writer.uint32(/* id 2, wireType 2 =*/18).string(message.timestamp);
            if (message.imageData != null && Object.hasOwnProperty.call(message, "imageData"))
                writer.uint32(/* id 3, wireType 2 =*/26).bytes(message.imageData);
            if (message.mimeType != null && Object.hasOwnProperty.call(message, "mimeType"))
                writer.uint32(/* id 4, wireType 2 =*/34).string(message.mimeType);
            if (message.metadata != null && Object.hasOwnProperty.call(message, "metadata"))
                for (let keys = Object.keys(message.metadata), i = 0; i < keys.length; ++i)
                    writer.uint32(/* id 5, wireType 2 =*/42).fork().uint32(/* id 1, wireType 2 =*/10).string(keys[i]).uint32(/* id 2, wireType 2 =*/18).string(message.metadata[keys[i]]).ldelim();
            return writer;
        };

        /**
         * Encodes the specified ScreenshotData message, length delimited. Does not implicitly {@link lifelog.ScreenshotData.verify|verify} messages.
         * @function encodeDelimited
         * @memberof lifelog.ScreenshotData
         * @static
         * @param {lifelog.IScreenshotData} message ScreenshotData message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        ScreenshotData.encodeDelimited = function encodeDelimited(message, writer) {
            return this.encode(message, writer).ldelim();
        };

        /**
         * Decodes a ScreenshotData message from the specified reader or buffer.
         * @function decode
         * @memberof lifelog.ScreenshotData
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @param {number} [length] Message length if known beforehand
         * @returns {lifelog.ScreenshotData} ScreenshotData
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        ScreenshotData.decode = function decode(reader, length, error) {
            if (!(reader instanceof $Reader))
                reader = $Reader.create(reader);
            let end = length === undefined ? reader.len : reader.pos + length, message = new $root.lifelog.ScreenshotData(), key, value;
            while (reader.pos < end) {
                let tag = reader.uint32();
                if (tag === error)
                    break;
                switch (tag >>> 3) {
                case 1: {
                        message.id = reader.string();
                        break;
                    }
                case 2: {
                        message.timestamp = reader.string();
                        break;
                    }
                case 3: {
                        message.imageData = reader.bytes();
                        break;
                    }
                case 4: {
                        message.mimeType = reader.string();
                        break;
                    }
                case 5: {
                        if (message.metadata === $util.emptyObject)
                            message.metadata = {};
                        let end2 = reader.uint32() + reader.pos;
                        key = "";
                        value = "";
                        while (reader.pos < end2) {
                            let tag2 = reader.uint32();
                            switch (tag2 >>> 3) {
                            case 1:
                                key = reader.string();
                                break;
                            case 2:
                                value = reader.string();
                                break;
                            default:
                                reader.skipType(tag2 & 7);
                                break;
                            }
                        }
                        message.metadata[key] = value;
                        break;
                    }
                default:
                    reader.skipType(tag & 7);
                    break;
                }
            }
            return message;
        };

        /**
         * Decodes a ScreenshotData message from the specified reader or buffer, length delimited.
         * @function decodeDelimited
         * @memberof lifelog.ScreenshotData
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @returns {lifelog.ScreenshotData} ScreenshotData
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        ScreenshotData.decodeDelimited = function decodeDelimited(reader) {
            if (!(reader instanceof $Reader))
                reader = new $Reader(reader);
            return this.decode(reader, reader.uint32());
        };

        /**
         * Verifies a ScreenshotData message.
         * @function verify
         * @memberof lifelog.ScreenshotData
         * @static
         * @param {Object.<string,*>} message Plain object to verify
         * @returns {string|null} `null` if valid, otherwise the reason why it is not
         */
        ScreenshotData.verify = function verify(message) {
            if (typeof message !== "object" || message === null)
                return "object expected";
            if (message.id != null && message.hasOwnProperty("id"))
                if (!$util.isString(message.id))
                    return "id: string expected";
            if (message.timestamp != null && message.hasOwnProperty("timestamp"))
                if (!$util.isString(message.timestamp))
                    return "timestamp: string expected";
            if (message.imageData != null && message.hasOwnProperty("imageData"))
                if (!(message.imageData && typeof message.imageData.length === "number" || $util.isString(message.imageData)))
                    return "imageData: buffer expected";
            if (message.mimeType != null && message.hasOwnProperty("mimeType"))
                if (!$util.isString(message.mimeType))
                    return "mimeType: string expected";
            if (message.metadata != null && message.hasOwnProperty("metadata")) {
                if (!$util.isObject(message.metadata))
                    return "metadata: object expected";
                let key = Object.keys(message.metadata);
                for (let i = 0; i < key.length; ++i)
                    if (!$util.isString(message.metadata[key[i]]))
                        return "metadata: string{k:string} expected";
            }
            return null;
        };

        /**
         * Creates a ScreenshotData message from a plain object. Also converts values to their respective internal types.
         * @function fromObject
         * @memberof lifelog.ScreenshotData
         * @static
         * @param {Object.<string,*>} object Plain object
         * @returns {lifelog.ScreenshotData} ScreenshotData
         */
        ScreenshotData.fromObject = function fromObject(object) {
            if (object instanceof $root.lifelog.ScreenshotData)
                return object;
            let message = new $root.lifelog.ScreenshotData();
            if (object.id != null)
                message.id = String(object.id);
            if (object.timestamp != null)
                message.timestamp = String(object.timestamp);
            if (object.imageData != null)
                if (typeof object.imageData === "string")
                    $util.base64.decode(object.imageData, message.imageData = $util.newBuffer($util.base64.length(object.imageData)), 0);
                else if (object.imageData.length >= 0)
                    message.imageData = object.imageData;
            if (object.mimeType != null)
                message.mimeType = String(object.mimeType);
            if (object.metadata) {
                if (typeof object.metadata !== "object")
                    throw TypeError(".lifelog.ScreenshotData.metadata: object expected");
                message.metadata = {};
                for (let keys = Object.keys(object.metadata), i = 0; i < keys.length; ++i)
                    message.metadata[keys[i]] = String(object.metadata[keys[i]]);
            }
            return message;
        };

        /**
         * Creates a plain object from a ScreenshotData message. Also converts values to other types if specified.
         * @function toObject
         * @memberof lifelog.ScreenshotData
         * @static
         * @param {lifelog.ScreenshotData} message ScreenshotData
         * @param {$protobuf.IConversionOptions} [options] Conversion options
         * @returns {Object.<string,*>} Plain object
         */
        ScreenshotData.toObject = function toObject(message, options) {
            if (!options)
                options = {};
            let object = {};
            if (options.objects || options.defaults)
                object.metadata = {};
            if (options.defaults) {
                object.id = "";
                object.timestamp = "";
                if (options.bytes === String)
                    object.imageData = "";
                else {
                    object.imageData = [];
                    if (options.bytes !== Array)
                        object.imageData = $util.newBuffer(object.imageData);
                }
                object.mimeType = "";
            }
            if (message.id != null && message.hasOwnProperty("id"))
                object.id = message.id;
            if (message.timestamp != null && message.hasOwnProperty("timestamp"))
                object.timestamp = message.timestamp;
            if (message.imageData != null && message.hasOwnProperty("imageData"))
                object.imageData = options.bytes === String ? $util.base64.encode(message.imageData, 0, message.imageData.length) : options.bytes === Array ? Array.prototype.slice.call(message.imageData) : message.imageData;
            if (message.mimeType != null && message.hasOwnProperty("mimeType"))
                object.mimeType = message.mimeType;
            let keys2;
            if (message.metadata && (keys2 = Object.keys(message.metadata)).length) {
                object.metadata = {};
                for (let j = 0; j < keys2.length; ++j)
                    object.metadata[keys2[j]] = message.metadata[keys2[j]];
            }
            return object;
        };

        /**
         * Converts this ScreenshotData to JSON.
         * @function toJSON
         * @memberof lifelog.ScreenshotData
         * @instance
         * @returns {Object.<string,*>} JSON object
         */
        ScreenshotData.prototype.toJSON = function toJSON() {
            return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
        };

        /**
         * Gets the default type url for ScreenshotData
         * @function getTypeUrl
         * @memberof lifelog.ScreenshotData
         * @static
         * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns {string} The default type url
         */
        ScreenshotData.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
            if (typeUrlPrefix === undefined) {
                typeUrlPrefix = "type.googleapis.com";
            }
            return typeUrlPrefix + "/lifelog.ScreenshotData";
        };

        return ScreenshotData;
    })();

    lifelog.ProcessData = (function() {

        /**
         * Properties of a ProcessData.
         * @memberof lifelog
         * @interface IProcessData
         * @property {string|null} [id] ProcessData id
         * @property {string|null} [timestamp] ProcessData timestamp
         * @property {string|null} [processName] ProcessData processName
         * @property {string|null} [windowTitle] ProcessData windowTitle
         * @property {number|null} [pid] ProcessData pid
         * @property {number|null} [cpuUsage] ProcessData cpuUsage
         * @property {number|null} [memoryUsage] ProcessData memoryUsage
         * @property {boolean|null} [isFocused] ProcessData isFocused
         */

        /**
         * Constructs a new ProcessData.
         * @memberof lifelog
         * @classdesc Represents a ProcessData.
         * @implements IProcessData
         * @constructor
         * @param {lifelog.IProcessData=} [properties] Properties to set
         */
        function ProcessData(properties) {
            if (properties)
                for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                    if (properties[keys[i]] != null)
                        this[keys[i]] = properties[keys[i]];
        }

        /**
         * ProcessData id.
         * @member {string} id
         * @memberof lifelog.ProcessData
         * @instance
         */
        ProcessData.prototype.id = "";

        /**
         * ProcessData timestamp.
         * @member {string} timestamp
         * @memberof lifelog.ProcessData
         * @instance
         */
        ProcessData.prototype.timestamp = "";

        /**
         * ProcessData processName.
         * @member {string} processName
         * @memberof lifelog.ProcessData
         * @instance
         */
        ProcessData.prototype.processName = "";

        /**
         * ProcessData windowTitle.
         * @member {string} windowTitle
         * @memberof lifelog.ProcessData
         * @instance
         */
        ProcessData.prototype.windowTitle = "";

        /**
         * ProcessData pid.
         * @member {number} pid
         * @memberof lifelog.ProcessData
         * @instance
         */
        ProcessData.prototype.pid = 0;

        /**
         * ProcessData cpuUsage.
         * @member {number} cpuUsage
         * @memberof lifelog.ProcessData
         * @instance
         */
        ProcessData.prototype.cpuUsage = 0;

        /**
         * ProcessData memoryUsage.
         * @member {number} memoryUsage
         * @memberof lifelog.ProcessData
         * @instance
         */
        ProcessData.prototype.memoryUsage = 0;

        /**
         * ProcessData isFocused.
         * @member {boolean} isFocused
         * @memberof lifelog.ProcessData
         * @instance
         */
        ProcessData.prototype.isFocused = false;

        /**
         * Creates a new ProcessData instance using the specified properties.
         * @function create
         * @memberof lifelog.ProcessData
         * @static
         * @param {lifelog.IProcessData=} [properties] Properties to set
         * @returns {lifelog.ProcessData} ProcessData instance
         */
        ProcessData.create = function create(properties) {
            return new ProcessData(properties);
        };

        /**
         * Encodes the specified ProcessData message. Does not implicitly {@link lifelog.ProcessData.verify|verify} messages.
         * @function encode
         * @memberof lifelog.ProcessData
         * @static
         * @param {lifelog.IProcessData} message ProcessData message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        ProcessData.encode = function encode(message, writer) {
            if (!writer)
                writer = $Writer.create();
            if (message.id != null && Object.hasOwnProperty.call(message, "id"))
                writer.uint32(/* id 1, wireType 2 =*/10).string(message.id);
            if (message.timestamp != null && Object.hasOwnProperty.call(message, "timestamp"))
                writer.uint32(/* id 2, wireType 2 =*/18).string(message.timestamp);
            if (message.processName != null && Object.hasOwnProperty.call(message, "processName"))
                writer.uint32(/* id 3, wireType 2 =*/26).string(message.processName);
            if (message.windowTitle != null && Object.hasOwnProperty.call(message, "windowTitle"))
                writer.uint32(/* id 4, wireType 2 =*/34).string(message.windowTitle);
            if (message.pid != null && Object.hasOwnProperty.call(message, "pid"))
                writer.uint32(/* id 5, wireType 0 =*/40).int32(message.pid);
            if (message.cpuUsage != null && Object.hasOwnProperty.call(message, "cpuUsage"))
                writer.uint32(/* id 6, wireType 5 =*/53).float(message.cpuUsage);
            if (message.memoryUsage != null && Object.hasOwnProperty.call(message, "memoryUsage"))
                writer.uint32(/* id 7, wireType 5 =*/61).float(message.memoryUsage);
            if (message.isFocused != null && Object.hasOwnProperty.call(message, "isFocused"))
                writer.uint32(/* id 8, wireType 0 =*/64).bool(message.isFocused);
            return writer;
        };

        /**
         * Encodes the specified ProcessData message, length delimited. Does not implicitly {@link lifelog.ProcessData.verify|verify} messages.
         * @function encodeDelimited
         * @memberof lifelog.ProcessData
         * @static
         * @param {lifelog.IProcessData} message ProcessData message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        ProcessData.encodeDelimited = function encodeDelimited(message, writer) {
            return this.encode(message, writer).ldelim();
        };

        /**
         * Decodes a ProcessData message from the specified reader or buffer.
         * @function decode
         * @memberof lifelog.ProcessData
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @param {number} [length] Message length if known beforehand
         * @returns {lifelog.ProcessData} ProcessData
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        ProcessData.decode = function decode(reader, length, error) {
            if (!(reader instanceof $Reader))
                reader = $Reader.create(reader);
            let end = length === undefined ? reader.len : reader.pos + length, message = new $root.lifelog.ProcessData();
            while (reader.pos < end) {
                let tag = reader.uint32();
                if (tag === error)
                    break;
                switch (tag >>> 3) {
                case 1: {
                        message.id = reader.string();
                        break;
                    }
                case 2: {
                        message.timestamp = reader.string();
                        break;
                    }
                case 3: {
                        message.processName = reader.string();
                        break;
                    }
                case 4: {
                        message.windowTitle = reader.string();
                        break;
                    }
                case 5: {
                        message.pid = reader.int32();
                        break;
                    }
                case 6: {
                        message.cpuUsage = reader.float();
                        break;
                    }
                case 7: {
                        message.memoryUsage = reader.float();
                        break;
                    }
                case 8: {
                        message.isFocused = reader.bool();
                        break;
                    }
                default:
                    reader.skipType(tag & 7);
                    break;
                }
            }
            return message;
        };

        /**
         * Decodes a ProcessData message from the specified reader or buffer, length delimited.
         * @function decodeDelimited
         * @memberof lifelog.ProcessData
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @returns {lifelog.ProcessData} ProcessData
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        ProcessData.decodeDelimited = function decodeDelimited(reader) {
            if (!(reader instanceof $Reader))
                reader = new $Reader(reader);
            return this.decode(reader, reader.uint32());
        };

        /**
         * Verifies a ProcessData message.
         * @function verify
         * @memberof lifelog.ProcessData
         * @static
         * @param {Object.<string,*>} message Plain object to verify
         * @returns {string|null} `null` if valid, otherwise the reason why it is not
         */
        ProcessData.verify = function verify(message) {
            if (typeof message !== "object" || message === null)
                return "object expected";
            if (message.id != null && message.hasOwnProperty("id"))
                if (!$util.isString(message.id))
                    return "id: string expected";
            if (message.timestamp != null && message.hasOwnProperty("timestamp"))
                if (!$util.isString(message.timestamp))
                    return "timestamp: string expected";
            if (message.processName != null && message.hasOwnProperty("processName"))
                if (!$util.isString(message.processName))
                    return "processName: string expected";
            if (message.windowTitle != null && message.hasOwnProperty("windowTitle"))
                if (!$util.isString(message.windowTitle))
                    return "windowTitle: string expected";
            if (message.pid != null && message.hasOwnProperty("pid"))
                if (!$util.isInteger(message.pid))
                    return "pid: integer expected";
            if (message.cpuUsage != null && message.hasOwnProperty("cpuUsage"))
                if (typeof message.cpuUsage !== "number")
                    return "cpuUsage: number expected";
            if (message.memoryUsage != null && message.hasOwnProperty("memoryUsage"))
                if (typeof message.memoryUsage !== "number")
                    return "memoryUsage: number expected";
            if (message.isFocused != null && message.hasOwnProperty("isFocused"))
                if (typeof message.isFocused !== "boolean")
                    return "isFocused: boolean expected";
            return null;
        };

        /**
         * Creates a ProcessData message from a plain object. Also converts values to their respective internal types.
         * @function fromObject
         * @memberof lifelog.ProcessData
         * @static
         * @param {Object.<string,*>} object Plain object
         * @returns {lifelog.ProcessData} ProcessData
         */
        ProcessData.fromObject = function fromObject(object) {
            if (object instanceof $root.lifelog.ProcessData)
                return object;
            let message = new $root.lifelog.ProcessData();
            if (object.id != null)
                message.id = String(object.id);
            if (object.timestamp != null)
                message.timestamp = String(object.timestamp);
            if (object.processName != null)
                message.processName = String(object.processName);
            if (object.windowTitle != null)
                message.windowTitle = String(object.windowTitle);
            if (object.pid != null)
                message.pid = object.pid | 0;
            if (object.cpuUsage != null)
                message.cpuUsage = Number(object.cpuUsage);
            if (object.memoryUsage != null)
                message.memoryUsage = Number(object.memoryUsage);
            if (object.isFocused != null)
                message.isFocused = Boolean(object.isFocused);
            return message;
        };

        /**
         * Creates a plain object from a ProcessData message. Also converts values to other types if specified.
         * @function toObject
         * @memberof lifelog.ProcessData
         * @static
         * @param {lifelog.ProcessData} message ProcessData
         * @param {$protobuf.IConversionOptions} [options] Conversion options
         * @returns {Object.<string,*>} Plain object
         */
        ProcessData.toObject = function toObject(message, options) {
            if (!options)
                options = {};
            let object = {};
            if (options.defaults) {
                object.id = "";
                object.timestamp = "";
                object.processName = "";
                object.windowTitle = "";
                object.pid = 0;
                object.cpuUsage = 0;
                object.memoryUsage = 0;
                object.isFocused = false;
            }
            if (message.id != null && message.hasOwnProperty("id"))
                object.id = message.id;
            if (message.timestamp != null && message.hasOwnProperty("timestamp"))
                object.timestamp = message.timestamp;
            if (message.processName != null && message.hasOwnProperty("processName"))
                object.processName = message.processName;
            if (message.windowTitle != null && message.hasOwnProperty("windowTitle"))
                object.windowTitle = message.windowTitle;
            if (message.pid != null && message.hasOwnProperty("pid"))
                object.pid = message.pid;
            if (message.cpuUsage != null && message.hasOwnProperty("cpuUsage"))
                object.cpuUsage = options.json && !isFinite(message.cpuUsage) ? String(message.cpuUsage) : message.cpuUsage;
            if (message.memoryUsage != null && message.hasOwnProperty("memoryUsage"))
                object.memoryUsage = options.json && !isFinite(message.memoryUsage) ? String(message.memoryUsage) : message.memoryUsage;
            if (message.isFocused != null && message.hasOwnProperty("isFocused"))
                object.isFocused = message.isFocused;
            return object;
        };

        /**
         * Converts this ProcessData to JSON.
         * @function toJSON
         * @memberof lifelog.ProcessData
         * @instance
         * @returns {Object.<string,*>} JSON object
         */
        ProcessData.prototype.toJSON = function toJSON() {
            return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
        };

        /**
         * Gets the default type url for ProcessData
         * @function getTypeUrl
         * @memberof lifelog.ProcessData
         * @static
         * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns {string} The default type url
         */
        ProcessData.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
            if (typeUrlPrefix === undefined) {
                typeUrlPrefix = "type.googleapis.com";
            }
            return typeUrlPrefix + "/lifelog.ProcessData";
        };

        return ProcessData;
    })();

    lifelog.ProcessStatsRequest = (function() {

        /**
         * Properties of a ProcessStatsRequest.
         * @memberof lifelog
         * @interface IProcessStatsRequest
         * @property {lifelog.ITimeRangeRequest|null} [timeRange] ProcessStatsRequest timeRange
         * @property {string|null} [processName] ProcessStatsRequest processName
         * @property {boolean|null} [aggregate] ProcessStatsRequest aggregate
         */

        /**
         * Constructs a new ProcessStatsRequest.
         * @memberof lifelog
         * @classdesc Represents a ProcessStatsRequest.
         * @implements IProcessStatsRequest
         * @constructor
         * @param {lifelog.IProcessStatsRequest=} [properties] Properties to set
         */
        function ProcessStatsRequest(properties) {
            if (properties)
                for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                    if (properties[keys[i]] != null)
                        this[keys[i]] = properties[keys[i]];
        }

        /**
         * ProcessStatsRequest timeRange.
         * @member {lifelog.ITimeRangeRequest|null|undefined} timeRange
         * @memberof lifelog.ProcessStatsRequest
         * @instance
         */
        ProcessStatsRequest.prototype.timeRange = null;

        /**
         * ProcessStatsRequest processName.
         * @member {string} processName
         * @memberof lifelog.ProcessStatsRequest
         * @instance
         */
        ProcessStatsRequest.prototype.processName = "";

        /**
         * ProcessStatsRequest aggregate.
         * @member {boolean} aggregate
         * @memberof lifelog.ProcessStatsRequest
         * @instance
         */
        ProcessStatsRequest.prototype.aggregate = false;

        /**
         * Creates a new ProcessStatsRequest instance using the specified properties.
         * @function create
         * @memberof lifelog.ProcessStatsRequest
         * @static
         * @param {lifelog.IProcessStatsRequest=} [properties] Properties to set
         * @returns {lifelog.ProcessStatsRequest} ProcessStatsRequest instance
         */
        ProcessStatsRequest.create = function create(properties) {
            return new ProcessStatsRequest(properties);
        };

        /**
         * Encodes the specified ProcessStatsRequest message. Does not implicitly {@link lifelog.ProcessStatsRequest.verify|verify} messages.
         * @function encode
         * @memberof lifelog.ProcessStatsRequest
         * @static
         * @param {lifelog.IProcessStatsRequest} message ProcessStatsRequest message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        ProcessStatsRequest.encode = function encode(message, writer) {
            if (!writer)
                writer = $Writer.create();
            if (message.timeRange != null && Object.hasOwnProperty.call(message, "timeRange"))
                $root.lifelog.TimeRangeRequest.encode(message.timeRange, writer.uint32(/* id 1, wireType 2 =*/10).fork()).ldelim();
            if (message.processName != null && Object.hasOwnProperty.call(message, "processName"))
                writer.uint32(/* id 2, wireType 2 =*/18).string(message.processName);
            if (message.aggregate != null && Object.hasOwnProperty.call(message, "aggregate"))
                writer.uint32(/* id 3, wireType 0 =*/24).bool(message.aggregate);
            return writer;
        };

        /**
         * Encodes the specified ProcessStatsRequest message, length delimited. Does not implicitly {@link lifelog.ProcessStatsRequest.verify|verify} messages.
         * @function encodeDelimited
         * @memberof lifelog.ProcessStatsRequest
         * @static
         * @param {lifelog.IProcessStatsRequest} message ProcessStatsRequest message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        ProcessStatsRequest.encodeDelimited = function encodeDelimited(message, writer) {
            return this.encode(message, writer).ldelim();
        };

        /**
         * Decodes a ProcessStatsRequest message from the specified reader or buffer.
         * @function decode
         * @memberof lifelog.ProcessStatsRequest
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @param {number} [length] Message length if known beforehand
         * @returns {lifelog.ProcessStatsRequest} ProcessStatsRequest
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        ProcessStatsRequest.decode = function decode(reader, length, error) {
            if (!(reader instanceof $Reader))
                reader = $Reader.create(reader);
            let end = length === undefined ? reader.len : reader.pos + length, message = new $root.lifelog.ProcessStatsRequest();
            while (reader.pos < end) {
                let tag = reader.uint32();
                if (tag === error)
                    break;
                switch (tag >>> 3) {
                case 1: {
                        message.timeRange = $root.lifelog.TimeRangeRequest.decode(reader, reader.uint32());
                        break;
                    }
                case 2: {
                        message.processName = reader.string();
                        break;
                    }
                case 3: {
                        message.aggregate = reader.bool();
                        break;
                    }
                default:
                    reader.skipType(tag & 7);
                    break;
                }
            }
            return message;
        };

        /**
         * Decodes a ProcessStatsRequest message from the specified reader or buffer, length delimited.
         * @function decodeDelimited
         * @memberof lifelog.ProcessStatsRequest
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @returns {lifelog.ProcessStatsRequest} ProcessStatsRequest
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        ProcessStatsRequest.decodeDelimited = function decodeDelimited(reader) {
            if (!(reader instanceof $Reader))
                reader = new $Reader(reader);
            return this.decode(reader, reader.uint32());
        };

        /**
         * Verifies a ProcessStatsRequest message.
         * @function verify
         * @memberof lifelog.ProcessStatsRequest
         * @static
         * @param {Object.<string,*>} message Plain object to verify
         * @returns {string|null} `null` if valid, otherwise the reason why it is not
         */
        ProcessStatsRequest.verify = function verify(message) {
            if (typeof message !== "object" || message === null)
                return "object expected";
            if (message.timeRange != null && message.hasOwnProperty("timeRange")) {
                let error = $root.lifelog.TimeRangeRequest.verify(message.timeRange);
                if (error)
                    return "timeRange." + error;
            }
            if (message.processName != null && message.hasOwnProperty("processName"))
                if (!$util.isString(message.processName))
                    return "processName: string expected";
            if (message.aggregate != null && message.hasOwnProperty("aggregate"))
                if (typeof message.aggregate !== "boolean")
                    return "aggregate: boolean expected";
            return null;
        };

        /**
         * Creates a ProcessStatsRequest message from a plain object. Also converts values to their respective internal types.
         * @function fromObject
         * @memberof lifelog.ProcessStatsRequest
         * @static
         * @param {Object.<string,*>} object Plain object
         * @returns {lifelog.ProcessStatsRequest} ProcessStatsRequest
         */
        ProcessStatsRequest.fromObject = function fromObject(object) {
            if (object instanceof $root.lifelog.ProcessStatsRequest)
                return object;
            let message = new $root.lifelog.ProcessStatsRequest();
            if (object.timeRange != null) {
                if (typeof object.timeRange !== "object")
                    throw TypeError(".lifelog.ProcessStatsRequest.timeRange: object expected");
                message.timeRange = $root.lifelog.TimeRangeRequest.fromObject(object.timeRange);
            }
            if (object.processName != null)
                message.processName = String(object.processName);
            if (object.aggregate != null)
                message.aggregate = Boolean(object.aggregate);
            return message;
        };

        /**
         * Creates a plain object from a ProcessStatsRequest message. Also converts values to other types if specified.
         * @function toObject
         * @memberof lifelog.ProcessStatsRequest
         * @static
         * @param {lifelog.ProcessStatsRequest} message ProcessStatsRequest
         * @param {$protobuf.IConversionOptions} [options] Conversion options
         * @returns {Object.<string,*>} Plain object
         */
        ProcessStatsRequest.toObject = function toObject(message, options) {
            if (!options)
                options = {};
            let object = {};
            if (options.defaults) {
                object.timeRange = null;
                object.processName = "";
                object.aggregate = false;
            }
            if (message.timeRange != null && message.hasOwnProperty("timeRange"))
                object.timeRange = $root.lifelog.TimeRangeRequest.toObject(message.timeRange, options);
            if (message.processName != null && message.hasOwnProperty("processName"))
                object.processName = message.processName;
            if (message.aggregate != null && message.hasOwnProperty("aggregate"))
                object.aggregate = message.aggregate;
            return object;
        };

        /**
         * Converts this ProcessStatsRequest to JSON.
         * @function toJSON
         * @memberof lifelog.ProcessStatsRequest
         * @instance
         * @returns {Object.<string,*>} JSON object
         */
        ProcessStatsRequest.prototype.toJSON = function toJSON() {
            return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
        };

        /**
         * Gets the default type url for ProcessStatsRequest
         * @function getTypeUrl
         * @memberof lifelog.ProcessStatsRequest
         * @static
         * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns {string} The default type url
         */
        ProcessStatsRequest.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
            if (typeUrlPrefix === undefined) {
                typeUrlPrefix = "type.googleapis.com";
            }
            return typeUrlPrefix + "/lifelog.ProcessStatsRequest";
        };

        return ProcessStatsRequest;
    })();

    lifelog.ProcessStatsSummary = (function() {

        /**
         * Properties of a ProcessStatsSummary.
         * @memberof lifelog
         * @interface IProcessStatsSummary
         * @property {string|null} [processName] ProcessStatsSummary processName
         * @property {number|null} [totalActiveTime] ProcessStatsSummary totalActiveTime
         * @property {number|null} [averageCpuUsage] ProcessStatsSummary averageCpuUsage
         * @property {number|null} [averageMemoryUsage] ProcessStatsSummary averageMemoryUsage
         * @property {number|null} [focusCount] ProcessStatsSummary focusCount
         */

        /**
         * Constructs a new ProcessStatsSummary.
         * @memberof lifelog
         * @classdesc Represents a ProcessStatsSummary.
         * @implements IProcessStatsSummary
         * @constructor
         * @param {lifelog.IProcessStatsSummary=} [properties] Properties to set
         */
        function ProcessStatsSummary(properties) {
            if (properties)
                for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                    if (properties[keys[i]] != null)
                        this[keys[i]] = properties[keys[i]];
        }

        /**
         * ProcessStatsSummary processName.
         * @member {string} processName
         * @memberof lifelog.ProcessStatsSummary
         * @instance
         */
        ProcessStatsSummary.prototype.processName = "";

        /**
         * ProcessStatsSummary totalActiveTime.
         * @member {number} totalActiveTime
         * @memberof lifelog.ProcessStatsSummary
         * @instance
         */
        ProcessStatsSummary.prototype.totalActiveTime = 0;

        /**
         * ProcessStatsSummary averageCpuUsage.
         * @member {number} averageCpuUsage
         * @memberof lifelog.ProcessStatsSummary
         * @instance
         */
        ProcessStatsSummary.prototype.averageCpuUsage = 0;

        /**
         * ProcessStatsSummary averageMemoryUsage.
         * @member {number} averageMemoryUsage
         * @memberof lifelog.ProcessStatsSummary
         * @instance
         */
        ProcessStatsSummary.prototype.averageMemoryUsage = 0;

        /**
         * ProcessStatsSummary focusCount.
         * @member {number} focusCount
         * @memberof lifelog.ProcessStatsSummary
         * @instance
         */
        ProcessStatsSummary.prototype.focusCount = 0;

        /**
         * Creates a new ProcessStatsSummary instance using the specified properties.
         * @function create
         * @memberof lifelog.ProcessStatsSummary
         * @static
         * @param {lifelog.IProcessStatsSummary=} [properties] Properties to set
         * @returns {lifelog.ProcessStatsSummary} ProcessStatsSummary instance
         */
        ProcessStatsSummary.create = function create(properties) {
            return new ProcessStatsSummary(properties);
        };

        /**
         * Encodes the specified ProcessStatsSummary message. Does not implicitly {@link lifelog.ProcessStatsSummary.verify|verify} messages.
         * @function encode
         * @memberof lifelog.ProcessStatsSummary
         * @static
         * @param {lifelog.IProcessStatsSummary} message ProcessStatsSummary message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        ProcessStatsSummary.encode = function encode(message, writer) {
            if (!writer)
                writer = $Writer.create();
            if (message.processName != null && Object.hasOwnProperty.call(message, "processName"))
                writer.uint32(/* id 1, wireType 2 =*/10).string(message.processName);
            if (message.totalActiveTime != null && Object.hasOwnProperty.call(message, "totalActiveTime"))
                writer.uint32(/* id 2, wireType 1 =*/17).double(message.totalActiveTime);
            if (message.averageCpuUsage != null && Object.hasOwnProperty.call(message, "averageCpuUsage"))
                writer.uint32(/* id 3, wireType 1 =*/25).double(message.averageCpuUsage);
            if (message.averageMemoryUsage != null && Object.hasOwnProperty.call(message, "averageMemoryUsage"))
                writer.uint32(/* id 4, wireType 1 =*/33).double(message.averageMemoryUsage);
            if (message.focusCount != null && Object.hasOwnProperty.call(message, "focusCount"))
                writer.uint32(/* id 5, wireType 0 =*/40).int32(message.focusCount);
            return writer;
        };

        /**
         * Encodes the specified ProcessStatsSummary message, length delimited. Does not implicitly {@link lifelog.ProcessStatsSummary.verify|verify} messages.
         * @function encodeDelimited
         * @memberof lifelog.ProcessStatsSummary
         * @static
         * @param {lifelog.IProcessStatsSummary} message ProcessStatsSummary message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        ProcessStatsSummary.encodeDelimited = function encodeDelimited(message, writer) {
            return this.encode(message, writer).ldelim();
        };

        /**
         * Decodes a ProcessStatsSummary message from the specified reader or buffer.
         * @function decode
         * @memberof lifelog.ProcessStatsSummary
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @param {number} [length] Message length if known beforehand
         * @returns {lifelog.ProcessStatsSummary} ProcessStatsSummary
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        ProcessStatsSummary.decode = function decode(reader, length, error) {
            if (!(reader instanceof $Reader))
                reader = $Reader.create(reader);
            let end = length === undefined ? reader.len : reader.pos + length, message = new $root.lifelog.ProcessStatsSummary();
            while (reader.pos < end) {
                let tag = reader.uint32();
                if (tag === error)
                    break;
                switch (tag >>> 3) {
                case 1: {
                        message.processName = reader.string();
                        break;
                    }
                case 2: {
                        message.totalActiveTime = reader.double();
                        break;
                    }
                case 3: {
                        message.averageCpuUsage = reader.double();
                        break;
                    }
                case 4: {
                        message.averageMemoryUsage = reader.double();
                        break;
                    }
                case 5: {
                        message.focusCount = reader.int32();
                        break;
                    }
                default:
                    reader.skipType(tag & 7);
                    break;
                }
            }
            return message;
        };

        /**
         * Decodes a ProcessStatsSummary message from the specified reader or buffer, length delimited.
         * @function decodeDelimited
         * @memberof lifelog.ProcessStatsSummary
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @returns {lifelog.ProcessStatsSummary} ProcessStatsSummary
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        ProcessStatsSummary.decodeDelimited = function decodeDelimited(reader) {
            if (!(reader instanceof $Reader))
                reader = new $Reader(reader);
            return this.decode(reader, reader.uint32());
        };

        /**
         * Verifies a ProcessStatsSummary message.
         * @function verify
         * @memberof lifelog.ProcessStatsSummary
         * @static
         * @param {Object.<string,*>} message Plain object to verify
         * @returns {string|null} `null` if valid, otherwise the reason why it is not
         */
        ProcessStatsSummary.verify = function verify(message) {
            if (typeof message !== "object" || message === null)
                return "object expected";
            if (message.processName != null && message.hasOwnProperty("processName"))
                if (!$util.isString(message.processName))
                    return "processName: string expected";
            if (message.totalActiveTime != null && message.hasOwnProperty("totalActiveTime"))
                if (typeof message.totalActiveTime !== "number")
                    return "totalActiveTime: number expected";
            if (message.averageCpuUsage != null && message.hasOwnProperty("averageCpuUsage"))
                if (typeof message.averageCpuUsage !== "number")
                    return "averageCpuUsage: number expected";
            if (message.averageMemoryUsage != null && message.hasOwnProperty("averageMemoryUsage"))
                if (typeof message.averageMemoryUsage !== "number")
                    return "averageMemoryUsage: number expected";
            if (message.focusCount != null && message.hasOwnProperty("focusCount"))
                if (!$util.isInteger(message.focusCount))
                    return "focusCount: integer expected";
            return null;
        };

        /**
         * Creates a ProcessStatsSummary message from a plain object. Also converts values to their respective internal types.
         * @function fromObject
         * @memberof lifelog.ProcessStatsSummary
         * @static
         * @param {Object.<string,*>} object Plain object
         * @returns {lifelog.ProcessStatsSummary} ProcessStatsSummary
         */
        ProcessStatsSummary.fromObject = function fromObject(object) {
            if (object instanceof $root.lifelog.ProcessStatsSummary)
                return object;
            let message = new $root.lifelog.ProcessStatsSummary();
            if (object.processName != null)
                message.processName = String(object.processName);
            if (object.totalActiveTime != null)
                message.totalActiveTime = Number(object.totalActiveTime);
            if (object.averageCpuUsage != null)
                message.averageCpuUsage = Number(object.averageCpuUsage);
            if (object.averageMemoryUsage != null)
                message.averageMemoryUsage = Number(object.averageMemoryUsage);
            if (object.focusCount != null)
                message.focusCount = object.focusCount | 0;
            return message;
        };

        /**
         * Creates a plain object from a ProcessStatsSummary message. Also converts values to other types if specified.
         * @function toObject
         * @memberof lifelog.ProcessStatsSummary
         * @static
         * @param {lifelog.ProcessStatsSummary} message ProcessStatsSummary
         * @param {$protobuf.IConversionOptions} [options] Conversion options
         * @returns {Object.<string,*>} Plain object
         */
        ProcessStatsSummary.toObject = function toObject(message, options) {
            if (!options)
                options = {};
            let object = {};
            if (options.defaults) {
                object.processName = "";
                object.totalActiveTime = 0;
                object.averageCpuUsage = 0;
                object.averageMemoryUsage = 0;
                object.focusCount = 0;
            }
            if (message.processName != null && message.hasOwnProperty("processName"))
                object.processName = message.processName;
            if (message.totalActiveTime != null && message.hasOwnProperty("totalActiveTime"))
                object.totalActiveTime = options.json && !isFinite(message.totalActiveTime) ? String(message.totalActiveTime) : message.totalActiveTime;
            if (message.averageCpuUsage != null && message.hasOwnProperty("averageCpuUsage"))
                object.averageCpuUsage = options.json && !isFinite(message.averageCpuUsage) ? String(message.averageCpuUsage) : message.averageCpuUsage;
            if (message.averageMemoryUsage != null && message.hasOwnProperty("averageMemoryUsage"))
                object.averageMemoryUsage = options.json && !isFinite(message.averageMemoryUsage) ? String(message.averageMemoryUsage) : message.averageMemoryUsage;
            if (message.focusCount != null && message.hasOwnProperty("focusCount"))
                object.focusCount = message.focusCount;
            return object;
        };

        /**
         * Converts this ProcessStatsSummary to JSON.
         * @function toJSON
         * @memberof lifelog.ProcessStatsSummary
         * @instance
         * @returns {Object.<string,*>} JSON object
         */
        ProcessStatsSummary.prototype.toJSON = function toJSON() {
            return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
        };

        /**
         * Gets the default type url for ProcessStatsSummary
         * @function getTypeUrl
         * @memberof lifelog.ProcessStatsSummary
         * @static
         * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns {string} The default type url
         */
        ProcessStatsSummary.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
            if (typeUrlPrefix === undefined) {
                typeUrlPrefix = "type.googleapis.com";
            }
            return typeUrlPrefix + "/lifelog.ProcessStatsSummary";
        };

        return ProcessStatsSummary;
    })();

    lifelog.ProcessStatsResponse = (function() {

        /**
         * Properties of a ProcessStatsResponse.
         * @memberof lifelog
         * @interface IProcessStatsResponse
         * @property {Array.<lifelog.IProcessStatsSummary>|null} [summaries] ProcessStatsResponse summaries
         * @property {Object.<string,number>|null} [usageByHour] ProcessStatsResponse usageByHour
         */

        /**
         * Constructs a new ProcessStatsResponse.
         * @memberof lifelog
         * @classdesc Represents a ProcessStatsResponse.
         * @implements IProcessStatsResponse
         * @constructor
         * @param {lifelog.IProcessStatsResponse=} [properties] Properties to set
         */
        function ProcessStatsResponse(properties) {
            this.summaries = [];
            this.usageByHour = {};
            if (properties)
                for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                    if (properties[keys[i]] != null)
                        this[keys[i]] = properties[keys[i]];
        }

        /**
         * ProcessStatsResponse summaries.
         * @member {Array.<lifelog.IProcessStatsSummary>} summaries
         * @memberof lifelog.ProcessStatsResponse
         * @instance
         */
        ProcessStatsResponse.prototype.summaries = $util.emptyArray;

        /**
         * ProcessStatsResponse usageByHour.
         * @member {Object.<string,number>} usageByHour
         * @memberof lifelog.ProcessStatsResponse
         * @instance
         */
        ProcessStatsResponse.prototype.usageByHour = $util.emptyObject;

        /**
         * Creates a new ProcessStatsResponse instance using the specified properties.
         * @function create
         * @memberof lifelog.ProcessStatsResponse
         * @static
         * @param {lifelog.IProcessStatsResponse=} [properties] Properties to set
         * @returns {lifelog.ProcessStatsResponse} ProcessStatsResponse instance
         */
        ProcessStatsResponse.create = function create(properties) {
            return new ProcessStatsResponse(properties);
        };

        /**
         * Encodes the specified ProcessStatsResponse message. Does not implicitly {@link lifelog.ProcessStatsResponse.verify|verify} messages.
         * @function encode
         * @memberof lifelog.ProcessStatsResponse
         * @static
         * @param {lifelog.IProcessStatsResponse} message ProcessStatsResponse message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        ProcessStatsResponse.encode = function encode(message, writer) {
            if (!writer)
                writer = $Writer.create();
            if (message.summaries != null && message.summaries.length)
                for (let i = 0; i < message.summaries.length; ++i)
                    $root.lifelog.ProcessStatsSummary.encode(message.summaries[i], writer.uint32(/* id 1, wireType 2 =*/10).fork()).ldelim();
            if (message.usageByHour != null && Object.hasOwnProperty.call(message, "usageByHour"))
                for (let keys = Object.keys(message.usageByHour), i = 0; i < keys.length; ++i)
                    writer.uint32(/* id 2, wireType 2 =*/18).fork().uint32(/* id 1, wireType 2 =*/10).string(keys[i]).uint32(/* id 2, wireType 1 =*/17).double(message.usageByHour[keys[i]]).ldelim();
            return writer;
        };

        /**
         * Encodes the specified ProcessStatsResponse message, length delimited. Does not implicitly {@link lifelog.ProcessStatsResponse.verify|verify} messages.
         * @function encodeDelimited
         * @memberof lifelog.ProcessStatsResponse
         * @static
         * @param {lifelog.IProcessStatsResponse} message ProcessStatsResponse message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        ProcessStatsResponse.encodeDelimited = function encodeDelimited(message, writer) {
            return this.encode(message, writer).ldelim();
        };

        /**
         * Decodes a ProcessStatsResponse message from the specified reader or buffer.
         * @function decode
         * @memberof lifelog.ProcessStatsResponse
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @param {number} [length] Message length if known beforehand
         * @returns {lifelog.ProcessStatsResponse} ProcessStatsResponse
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        ProcessStatsResponse.decode = function decode(reader, length, error) {
            if (!(reader instanceof $Reader))
                reader = $Reader.create(reader);
            let end = length === undefined ? reader.len : reader.pos + length, message = new $root.lifelog.ProcessStatsResponse(), key, value;
            while (reader.pos < end) {
                let tag = reader.uint32();
                if (tag === error)
                    break;
                switch (tag >>> 3) {
                case 1: {
                        if (!(message.summaries && message.summaries.length))
                            message.summaries = [];
                        message.summaries.push($root.lifelog.ProcessStatsSummary.decode(reader, reader.uint32()));
                        break;
                    }
                case 2: {
                        if (message.usageByHour === $util.emptyObject)
                            message.usageByHour = {};
                        let end2 = reader.uint32() + reader.pos;
                        key = "";
                        value = 0;
                        while (reader.pos < end2) {
                            let tag2 = reader.uint32();
                            switch (tag2 >>> 3) {
                            case 1:
                                key = reader.string();
                                break;
                            case 2:
                                value = reader.double();
                                break;
                            default:
                                reader.skipType(tag2 & 7);
                                break;
                            }
                        }
                        message.usageByHour[key] = value;
                        break;
                    }
                default:
                    reader.skipType(tag & 7);
                    break;
                }
            }
            return message;
        };

        /**
         * Decodes a ProcessStatsResponse message from the specified reader or buffer, length delimited.
         * @function decodeDelimited
         * @memberof lifelog.ProcessStatsResponse
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @returns {lifelog.ProcessStatsResponse} ProcessStatsResponse
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        ProcessStatsResponse.decodeDelimited = function decodeDelimited(reader) {
            if (!(reader instanceof $Reader))
                reader = new $Reader(reader);
            return this.decode(reader, reader.uint32());
        };

        /**
         * Verifies a ProcessStatsResponse message.
         * @function verify
         * @memberof lifelog.ProcessStatsResponse
         * @static
         * @param {Object.<string,*>} message Plain object to verify
         * @returns {string|null} `null` if valid, otherwise the reason why it is not
         */
        ProcessStatsResponse.verify = function verify(message) {
            if (typeof message !== "object" || message === null)
                return "object expected";
            if (message.summaries != null && message.hasOwnProperty("summaries")) {
                if (!Array.isArray(message.summaries))
                    return "summaries: array expected";
                for (let i = 0; i < message.summaries.length; ++i) {
                    let error = $root.lifelog.ProcessStatsSummary.verify(message.summaries[i]);
                    if (error)
                        return "summaries." + error;
                }
            }
            if (message.usageByHour != null && message.hasOwnProperty("usageByHour")) {
                if (!$util.isObject(message.usageByHour))
                    return "usageByHour: object expected";
                let key = Object.keys(message.usageByHour);
                for (let i = 0; i < key.length; ++i)
                    if (typeof message.usageByHour[key[i]] !== "number")
                        return "usageByHour: number{k:string} expected";
            }
            return null;
        };

        /**
         * Creates a ProcessStatsResponse message from a plain object. Also converts values to their respective internal types.
         * @function fromObject
         * @memberof lifelog.ProcessStatsResponse
         * @static
         * @param {Object.<string,*>} object Plain object
         * @returns {lifelog.ProcessStatsResponse} ProcessStatsResponse
         */
        ProcessStatsResponse.fromObject = function fromObject(object) {
            if (object instanceof $root.lifelog.ProcessStatsResponse)
                return object;
            let message = new $root.lifelog.ProcessStatsResponse();
            if (object.summaries) {
                if (!Array.isArray(object.summaries))
                    throw TypeError(".lifelog.ProcessStatsResponse.summaries: array expected");
                message.summaries = [];
                for (let i = 0; i < object.summaries.length; ++i) {
                    if (typeof object.summaries[i] !== "object")
                        throw TypeError(".lifelog.ProcessStatsResponse.summaries: object expected");
                    message.summaries[i] = $root.lifelog.ProcessStatsSummary.fromObject(object.summaries[i]);
                }
            }
            if (object.usageByHour) {
                if (typeof object.usageByHour !== "object")
                    throw TypeError(".lifelog.ProcessStatsResponse.usageByHour: object expected");
                message.usageByHour = {};
                for (let keys = Object.keys(object.usageByHour), i = 0; i < keys.length; ++i)
                    message.usageByHour[keys[i]] = Number(object.usageByHour[keys[i]]);
            }
            return message;
        };

        /**
         * Creates a plain object from a ProcessStatsResponse message. Also converts values to other types if specified.
         * @function toObject
         * @memberof lifelog.ProcessStatsResponse
         * @static
         * @param {lifelog.ProcessStatsResponse} message ProcessStatsResponse
         * @param {$protobuf.IConversionOptions} [options] Conversion options
         * @returns {Object.<string,*>} Plain object
         */
        ProcessStatsResponse.toObject = function toObject(message, options) {
            if (!options)
                options = {};
            let object = {};
            if (options.arrays || options.defaults)
                object.summaries = [];
            if (options.objects || options.defaults)
                object.usageByHour = {};
            if (message.summaries && message.summaries.length) {
                object.summaries = [];
                for (let j = 0; j < message.summaries.length; ++j)
                    object.summaries[j] = $root.lifelog.ProcessStatsSummary.toObject(message.summaries[j], options);
            }
            let keys2;
            if (message.usageByHour && (keys2 = Object.keys(message.usageByHour)).length) {
                object.usageByHour = {};
                for (let j = 0; j < keys2.length; ++j)
                    object.usageByHour[keys2[j]] = options.json && !isFinite(message.usageByHour[keys2[j]]) ? String(message.usageByHour[keys2[j]]) : message.usageByHour[keys2[j]];
            }
            return object;
        };

        /**
         * Converts this ProcessStatsResponse to JSON.
         * @function toJSON
         * @memberof lifelog.ProcessStatsResponse
         * @instance
         * @returns {Object.<string,*>} JSON object
         */
        ProcessStatsResponse.prototype.toJSON = function toJSON() {
            return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
        };

        /**
         * Gets the default type url for ProcessStatsResponse
         * @function getTypeUrl
         * @memberof lifelog.ProcessStatsResponse
         * @static
         * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns {string} The default type url
         */
        ProcessStatsResponse.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
            if (typeUrlPrefix === undefined) {
                typeUrlPrefix = "type.googleapis.com";
            }
            return typeUrlPrefix + "/lifelog.ProcessStatsResponse";
        };

        return ProcessStatsResponse;
    })();

    lifelog.CameraFrameData = (function() {

        /**
         * Properties of a CameraFrameData.
         * @memberof lifelog
         * @interface ICameraFrameData
         * @property {string|null} [id] CameraFrameData id
         * @property {string|null} [timestamp] CameraFrameData timestamp
         * @property {Uint8Array|null} [imageData] CameraFrameData imageData
         * @property {string|null} [mimeType] CameraFrameData mimeType
         * @property {Object.<string,string>|null} [metadata] CameraFrameData metadata
         */

        /**
         * Constructs a new CameraFrameData.
         * @memberof lifelog
         * @classdesc Represents a CameraFrameData.
         * @implements ICameraFrameData
         * @constructor
         * @param {lifelog.ICameraFrameData=} [properties] Properties to set
         */
        function CameraFrameData(properties) {
            this.metadata = {};
            if (properties)
                for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                    if (properties[keys[i]] != null)
                        this[keys[i]] = properties[keys[i]];
        }

        /**
         * CameraFrameData id.
         * @member {string} id
         * @memberof lifelog.CameraFrameData
         * @instance
         */
        CameraFrameData.prototype.id = "";

        /**
         * CameraFrameData timestamp.
         * @member {string} timestamp
         * @memberof lifelog.CameraFrameData
         * @instance
         */
        CameraFrameData.prototype.timestamp = "";

        /**
         * CameraFrameData imageData.
         * @member {Uint8Array} imageData
         * @memberof lifelog.CameraFrameData
         * @instance
         */
        CameraFrameData.prototype.imageData = $util.newBuffer([]);

        /**
         * CameraFrameData mimeType.
         * @member {string} mimeType
         * @memberof lifelog.CameraFrameData
         * @instance
         */
        CameraFrameData.prototype.mimeType = "";

        /**
         * CameraFrameData metadata.
         * @member {Object.<string,string>} metadata
         * @memberof lifelog.CameraFrameData
         * @instance
         */
        CameraFrameData.prototype.metadata = $util.emptyObject;

        /**
         * Creates a new CameraFrameData instance using the specified properties.
         * @function create
         * @memberof lifelog.CameraFrameData
         * @static
         * @param {lifelog.ICameraFrameData=} [properties] Properties to set
         * @returns {lifelog.CameraFrameData} CameraFrameData instance
         */
        CameraFrameData.create = function create(properties) {
            return new CameraFrameData(properties);
        };

        /**
         * Encodes the specified CameraFrameData message. Does not implicitly {@link lifelog.CameraFrameData.verify|verify} messages.
         * @function encode
         * @memberof lifelog.CameraFrameData
         * @static
         * @param {lifelog.ICameraFrameData} message CameraFrameData message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        CameraFrameData.encode = function encode(message, writer) {
            if (!writer)
                writer = $Writer.create();
            if (message.id != null && Object.hasOwnProperty.call(message, "id"))
                writer.uint32(/* id 1, wireType 2 =*/10).string(message.id);
            if (message.timestamp != null && Object.hasOwnProperty.call(message, "timestamp"))
                writer.uint32(/* id 2, wireType 2 =*/18).string(message.timestamp);
            if (message.imageData != null && Object.hasOwnProperty.call(message, "imageData"))
                writer.uint32(/* id 3, wireType 2 =*/26).bytes(message.imageData);
            if (message.mimeType != null && Object.hasOwnProperty.call(message, "mimeType"))
                writer.uint32(/* id 4, wireType 2 =*/34).string(message.mimeType);
            if (message.metadata != null && Object.hasOwnProperty.call(message, "metadata"))
                for (let keys = Object.keys(message.metadata), i = 0; i < keys.length; ++i)
                    writer.uint32(/* id 5, wireType 2 =*/42).fork().uint32(/* id 1, wireType 2 =*/10).string(keys[i]).uint32(/* id 2, wireType 2 =*/18).string(message.metadata[keys[i]]).ldelim();
            return writer;
        };

        /**
         * Encodes the specified CameraFrameData message, length delimited. Does not implicitly {@link lifelog.CameraFrameData.verify|verify} messages.
         * @function encodeDelimited
         * @memberof lifelog.CameraFrameData
         * @static
         * @param {lifelog.ICameraFrameData} message CameraFrameData message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        CameraFrameData.encodeDelimited = function encodeDelimited(message, writer) {
            return this.encode(message, writer).ldelim();
        };

        /**
         * Decodes a CameraFrameData message from the specified reader or buffer.
         * @function decode
         * @memberof lifelog.CameraFrameData
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @param {number} [length] Message length if known beforehand
         * @returns {lifelog.CameraFrameData} CameraFrameData
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        CameraFrameData.decode = function decode(reader, length, error) {
            if (!(reader instanceof $Reader))
                reader = $Reader.create(reader);
            let end = length === undefined ? reader.len : reader.pos + length, message = new $root.lifelog.CameraFrameData(), key, value;
            while (reader.pos < end) {
                let tag = reader.uint32();
                if (tag === error)
                    break;
                switch (tag >>> 3) {
                case 1: {
                        message.id = reader.string();
                        break;
                    }
                case 2: {
                        message.timestamp = reader.string();
                        break;
                    }
                case 3: {
                        message.imageData = reader.bytes();
                        break;
                    }
                case 4: {
                        message.mimeType = reader.string();
                        break;
                    }
                case 5: {
                        if (message.metadata === $util.emptyObject)
                            message.metadata = {};
                        let end2 = reader.uint32() + reader.pos;
                        key = "";
                        value = "";
                        while (reader.pos < end2) {
                            let tag2 = reader.uint32();
                            switch (tag2 >>> 3) {
                            case 1:
                                key = reader.string();
                                break;
                            case 2:
                                value = reader.string();
                                break;
                            default:
                                reader.skipType(tag2 & 7);
                                break;
                            }
                        }
                        message.metadata[key] = value;
                        break;
                    }
                default:
                    reader.skipType(tag & 7);
                    break;
                }
            }
            return message;
        };

        /**
         * Decodes a CameraFrameData message from the specified reader or buffer, length delimited.
         * @function decodeDelimited
         * @memberof lifelog.CameraFrameData
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @returns {lifelog.CameraFrameData} CameraFrameData
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        CameraFrameData.decodeDelimited = function decodeDelimited(reader) {
            if (!(reader instanceof $Reader))
                reader = new $Reader(reader);
            return this.decode(reader, reader.uint32());
        };

        /**
         * Verifies a CameraFrameData message.
         * @function verify
         * @memberof lifelog.CameraFrameData
         * @static
         * @param {Object.<string,*>} message Plain object to verify
         * @returns {string|null} `null` if valid, otherwise the reason why it is not
         */
        CameraFrameData.verify = function verify(message) {
            if (typeof message !== "object" || message === null)
                return "object expected";
            if (message.id != null && message.hasOwnProperty("id"))
                if (!$util.isString(message.id))
                    return "id: string expected";
            if (message.timestamp != null && message.hasOwnProperty("timestamp"))
                if (!$util.isString(message.timestamp))
                    return "timestamp: string expected";
            if (message.imageData != null && message.hasOwnProperty("imageData"))
                if (!(message.imageData && typeof message.imageData.length === "number" || $util.isString(message.imageData)))
                    return "imageData: buffer expected";
            if (message.mimeType != null && message.hasOwnProperty("mimeType"))
                if (!$util.isString(message.mimeType))
                    return "mimeType: string expected";
            if (message.metadata != null && message.hasOwnProperty("metadata")) {
                if (!$util.isObject(message.metadata))
                    return "metadata: object expected";
                let key = Object.keys(message.metadata);
                for (let i = 0; i < key.length; ++i)
                    if (!$util.isString(message.metadata[key[i]]))
                        return "metadata: string{k:string} expected";
            }
            return null;
        };

        /**
         * Creates a CameraFrameData message from a plain object. Also converts values to their respective internal types.
         * @function fromObject
         * @memberof lifelog.CameraFrameData
         * @static
         * @param {Object.<string,*>} object Plain object
         * @returns {lifelog.CameraFrameData} CameraFrameData
         */
        CameraFrameData.fromObject = function fromObject(object) {
            if (object instanceof $root.lifelog.CameraFrameData)
                return object;
            let message = new $root.lifelog.CameraFrameData();
            if (object.id != null)
                message.id = String(object.id);
            if (object.timestamp != null)
                message.timestamp = String(object.timestamp);
            if (object.imageData != null)
                if (typeof object.imageData === "string")
                    $util.base64.decode(object.imageData, message.imageData = $util.newBuffer($util.base64.length(object.imageData)), 0);
                else if (object.imageData.length >= 0)
                    message.imageData = object.imageData;
            if (object.mimeType != null)
                message.mimeType = String(object.mimeType);
            if (object.metadata) {
                if (typeof object.metadata !== "object")
                    throw TypeError(".lifelog.CameraFrameData.metadata: object expected");
                message.metadata = {};
                for (let keys = Object.keys(object.metadata), i = 0; i < keys.length; ++i)
                    message.metadata[keys[i]] = String(object.metadata[keys[i]]);
            }
            return message;
        };

        /**
         * Creates a plain object from a CameraFrameData message. Also converts values to other types if specified.
         * @function toObject
         * @memberof lifelog.CameraFrameData
         * @static
         * @param {lifelog.CameraFrameData} message CameraFrameData
         * @param {$protobuf.IConversionOptions} [options] Conversion options
         * @returns {Object.<string,*>} Plain object
         */
        CameraFrameData.toObject = function toObject(message, options) {
            if (!options)
                options = {};
            let object = {};
            if (options.objects || options.defaults)
                object.metadata = {};
            if (options.defaults) {
                object.id = "";
                object.timestamp = "";
                if (options.bytes === String)
                    object.imageData = "";
                else {
                    object.imageData = [];
                    if (options.bytes !== Array)
                        object.imageData = $util.newBuffer(object.imageData);
                }
                object.mimeType = "";
            }
            if (message.id != null && message.hasOwnProperty("id"))
                object.id = message.id;
            if (message.timestamp != null && message.hasOwnProperty("timestamp"))
                object.timestamp = message.timestamp;
            if (message.imageData != null && message.hasOwnProperty("imageData"))
                object.imageData = options.bytes === String ? $util.base64.encode(message.imageData, 0, message.imageData.length) : options.bytes === Array ? Array.prototype.slice.call(message.imageData) : message.imageData;
            if (message.mimeType != null && message.hasOwnProperty("mimeType"))
                object.mimeType = message.mimeType;
            let keys2;
            if (message.metadata && (keys2 = Object.keys(message.metadata)).length) {
                object.metadata = {};
                for (let j = 0; j < keys2.length; ++j)
                    object.metadata[keys2[j]] = message.metadata[keys2[j]];
            }
            return object;
        };

        /**
         * Converts this CameraFrameData to JSON.
         * @function toJSON
         * @memberof lifelog.CameraFrameData
         * @instance
         * @returns {Object.<string,*>} JSON object
         */
        CameraFrameData.prototype.toJSON = function toJSON() {
            return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
        };

        /**
         * Gets the default type url for CameraFrameData
         * @function getTypeUrl
         * @memberof lifelog.CameraFrameData
         * @static
         * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns {string} The default type url
         */
        CameraFrameData.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
            if (typeUrlPrefix === undefined) {
                typeUrlPrefix = "type.googleapis.com";
            }
            return typeUrlPrefix + "/lifelog.CameraFrameData";
        };

        return CameraFrameData;
    })();

    lifelog.ActivitySummary = (function() {

        /**
         * Properties of an ActivitySummary.
         * @memberof lifelog
         * @interface IActivitySummary
         * @property {lifelog.ITimeRangeRequest|null} [timeRange] ActivitySummary timeRange
         * @property {Array.<lifelog.IActivityPeriod>|null} [activityPeriods] ActivitySummary activityPeriods
         * @property {Object.<string,number>|null} [appUsage] ActivitySummary appUsage
         * @property {number|null} [totalScreenshots] ActivitySummary totalScreenshots
         * @property {number|null} [totalCameraFrames] ActivitySummary totalCameraFrames
         * @property {Object.<string,number>|null} [totalByLogger] ActivitySummary totalByLogger
         */

        /**
         * Constructs a new ActivitySummary.
         * @memberof lifelog
         * @classdesc Represents an ActivitySummary.
         * @implements IActivitySummary
         * @constructor
         * @param {lifelog.IActivitySummary=} [properties] Properties to set
         */
        function ActivitySummary(properties) {
            this.activityPeriods = [];
            this.appUsage = {};
            this.totalByLogger = {};
            if (properties)
                for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                    if (properties[keys[i]] != null)
                        this[keys[i]] = properties[keys[i]];
        }

        /**
         * ActivitySummary timeRange.
         * @member {lifelog.ITimeRangeRequest|null|undefined} timeRange
         * @memberof lifelog.ActivitySummary
         * @instance
         */
        ActivitySummary.prototype.timeRange = null;

        /**
         * ActivitySummary activityPeriods.
         * @member {Array.<lifelog.IActivityPeriod>} activityPeriods
         * @memberof lifelog.ActivitySummary
         * @instance
         */
        ActivitySummary.prototype.activityPeriods = $util.emptyArray;

        /**
         * ActivitySummary appUsage.
         * @member {Object.<string,number>} appUsage
         * @memberof lifelog.ActivitySummary
         * @instance
         */
        ActivitySummary.prototype.appUsage = $util.emptyObject;

        /**
         * ActivitySummary totalScreenshots.
         * @member {number} totalScreenshots
         * @memberof lifelog.ActivitySummary
         * @instance
         */
        ActivitySummary.prototype.totalScreenshots = 0;

        /**
         * ActivitySummary totalCameraFrames.
         * @member {number} totalCameraFrames
         * @memberof lifelog.ActivitySummary
         * @instance
         */
        ActivitySummary.prototype.totalCameraFrames = 0;

        /**
         * ActivitySummary totalByLogger.
         * @member {Object.<string,number>} totalByLogger
         * @memberof lifelog.ActivitySummary
         * @instance
         */
        ActivitySummary.prototype.totalByLogger = $util.emptyObject;

        /**
         * Creates a new ActivitySummary instance using the specified properties.
         * @function create
         * @memberof lifelog.ActivitySummary
         * @static
         * @param {lifelog.IActivitySummary=} [properties] Properties to set
         * @returns {lifelog.ActivitySummary} ActivitySummary instance
         */
        ActivitySummary.create = function create(properties) {
            return new ActivitySummary(properties);
        };

        /**
         * Encodes the specified ActivitySummary message. Does not implicitly {@link lifelog.ActivitySummary.verify|verify} messages.
         * @function encode
         * @memberof lifelog.ActivitySummary
         * @static
         * @param {lifelog.IActivitySummary} message ActivitySummary message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        ActivitySummary.encode = function encode(message, writer) {
            if (!writer)
                writer = $Writer.create();
            if (message.timeRange != null && Object.hasOwnProperty.call(message, "timeRange"))
                $root.lifelog.TimeRangeRequest.encode(message.timeRange, writer.uint32(/* id 1, wireType 2 =*/10).fork()).ldelim();
            if (message.activityPeriods != null && message.activityPeriods.length)
                for (let i = 0; i < message.activityPeriods.length; ++i)
                    $root.lifelog.ActivityPeriod.encode(message.activityPeriods[i], writer.uint32(/* id 2, wireType 2 =*/18).fork()).ldelim();
            if (message.appUsage != null && Object.hasOwnProperty.call(message, "appUsage"))
                for (let keys = Object.keys(message.appUsage), i = 0; i < keys.length; ++i)
                    writer.uint32(/* id 3, wireType 2 =*/26).fork().uint32(/* id 1, wireType 2 =*/10).string(keys[i]).uint32(/* id 2, wireType 1 =*/17).double(message.appUsage[keys[i]]).ldelim();
            if (message.totalScreenshots != null && Object.hasOwnProperty.call(message, "totalScreenshots"))
                writer.uint32(/* id 4, wireType 0 =*/32).int32(message.totalScreenshots);
            if (message.totalCameraFrames != null && Object.hasOwnProperty.call(message, "totalCameraFrames"))
                writer.uint32(/* id 5, wireType 0 =*/40).int32(message.totalCameraFrames);
            if (message.totalByLogger != null && Object.hasOwnProperty.call(message, "totalByLogger"))
                for (let keys = Object.keys(message.totalByLogger), i = 0; i < keys.length; ++i)
                    writer.uint32(/* id 6, wireType 2 =*/50).fork().uint32(/* id 1, wireType 2 =*/10).string(keys[i]).uint32(/* id 2, wireType 0 =*/16).int32(message.totalByLogger[keys[i]]).ldelim();
            return writer;
        };

        /**
         * Encodes the specified ActivitySummary message, length delimited. Does not implicitly {@link lifelog.ActivitySummary.verify|verify} messages.
         * @function encodeDelimited
         * @memberof lifelog.ActivitySummary
         * @static
         * @param {lifelog.IActivitySummary} message ActivitySummary message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        ActivitySummary.encodeDelimited = function encodeDelimited(message, writer) {
            return this.encode(message, writer).ldelim();
        };

        /**
         * Decodes an ActivitySummary message from the specified reader or buffer.
         * @function decode
         * @memberof lifelog.ActivitySummary
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @param {number} [length] Message length if known beforehand
         * @returns {lifelog.ActivitySummary} ActivitySummary
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        ActivitySummary.decode = function decode(reader, length, error) {
            if (!(reader instanceof $Reader))
                reader = $Reader.create(reader);
            let end = length === undefined ? reader.len : reader.pos + length, message = new $root.lifelog.ActivitySummary(), key, value;
            while (reader.pos < end) {
                let tag = reader.uint32();
                if (tag === error)
                    break;
                switch (tag >>> 3) {
                case 1: {
                        message.timeRange = $root.lifelog.TimeRangeRequest.decode(reader, reader.uint32());
                        break;
                    }
                case 2: {
                        if (!(message.activityPeriods && message.activityPeriods.length))
                            message.activityPeriods = [];
                        message.activityPeriods.push($root.lifelog.ActivityPeriod.decode(reader, reader.uint32()));
                        break;
                    }
                case 3: {
                        if (message.appUsage === $util.emptyObject)
                            message.appUsage = {};
                        let end2 = reader.uint32() + reader.pos;
                        key = "";
                        value = 0;
                        while (reader.pos < end2) {
                            let tag2 = reader.uint32();
                            switch (tag2 >>> 3) {
                            case 1:
                                key = reader.string();
                                break;
                            case 2:
                                value = reader.double();
                                break;
                            default:
                                reader.skipType(tag2 & 7);
                                break;
                            }
                        }
                        message.appUsage[key] = value;
                        break;
                    }
                case 4: {
                        message.totalScreenshots = reader.int32();
                        break;
                    }
                case 5: {
                        message.totalCameraFrames = reader.int32();
                        break;
                    }
                case 6: {
                        if (message.totalByLogger === $util.emptyObject)
                            message.totalByLogger = {};
                        let end2 = reader.uint32() + reader.pos;
                        key = "";
                        value = 0;
                        while (reader.pos < end2) {
                            let tag2 = reader.uint32();
                            switch (tag2 >>> 3) {
                            case 1:
                                key = reader.string();
                                break;
                            case 2:
                                value = reader.int32();
                                break;
                            default:
                                reader.skipType(tag2 & 7);
                                break;
                            }
                        }
                        message.totalByLogger[key] = value;
                        break;
                    }
                default:
                    reader.skipType(tag & 7);
                    break;
                }
            }
            return message;
        };

        /**
         * Decodes an ActivitySummary message from the specified reader or buffer, length delimited.
         * @function decodeDelimited
         * @memberof lifelog.ActivitySummary
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @returns {lifelog.ActivitySummary} ActivitySummary
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        ActivitySummary.decodeDelimited = function decodeDelimited(reader) {
            if (!(reader instanceof $Reader))
                reader = new $Reader(reader);
            return this.decode(reader, reader.uint32());
        };

        /**
         * Verifies an ActivitySummary message.
         * @function verify
         * @memberof lifelog.ActivitySummary
         * @static
         * @param {Object.<string,*>} message Plain object to verify
         * @returns {string|null} `null` if valid, otherwise the reason why it is not
         */
        ActivitySummary.verify = function verify(message) {
            if (typeof message !== "object" || message === null)
                return "object expected";
            if (message.timeRange != null && message.hasOwnProperty("timeRange")) {
                let error = $root.lifelog.TimeRangeRequest.verify(message.timeRange);
                if (error)
                    return "timeRange." + error;
            }
            if (message.activityPeriods != null && message.hasOwnProperty("activityPeriods")) {
                if (!Array.isArray(message.activityPeriods))
                    return "activityPeriods: array expected";
                for (let i = 0; i < message.activityPeriods.length; ++i) {
                    let error = $root.lifelog.ActivityPeriod.verify(message.activityPeriods[i]);
                    if (error)
                        return "activityPeriods." + error;
                }
            }
            if (message.appUsage != null && message.hasOwnProperty("appUsage")) {
                if (!$util.isObject(message.appUsage))
                    return "appUsage: object expected";
                let key = Object.keys(message.appUsage);
                for (let i = 0; i < key.length; ++i)
                    if (typeof message.appUsage[key[i]] !== "number")
                        return "appUsage: number{k:string} expected";
            }
            if (message.totalScreenshots != null && message.hasOwnProperty("totalScreenshots"))
                if (!$util.isInteger(message.totalScreenshots))
                    return "totalScreenshots: integer expected";
            if (message.totalCameraFrames != null && message.hasOwnProperty("totalCameraFrames"))
                if (!$util.isInteger(message.totalCameraFrames))
                    return "totalCameraFrames: integer expected";
            if (message.totalByLogger != null && message.hasOwnProperty("totalByLogger")) {
                if (!$util.isObject(message.totalByLogger))
                    return "totalByLogger: object expected";
                let key = Object.keys(message.totalByLogger);
                for (let i = 0; i < key.length; ++i)
                    if (!$util.isInteger(message.totalByLogger[key[i]]))
                        return "totalByLogger: integer{k:string} expected";
            }
            return null;
        };

        /**
         * Creates an ActivitySummary message from a plain object. Also converts values to their respective internal types.
         * @function fromObject
         * @memberof lifelog.ActivitySummary
         * @static
         * @param {Object.<string,*>} object Plain object
         * @returns {lifelog.ActivitySummary} ActivitySummary
         */
        ActivitySummary.fromObject = function fromObject(object) {
            if (object instanceof $root.lifelog.ActivitySummary)
                return object;
            let message = new $root.lifelog.ActivitySummary();
            if (object.timeRange != null) {
                if (typeof object.timeRange !== "object")
                    throw TypeError(".lifelog.ActivitySummary.timeRange: object expected");
                message.timeRange = $root.lifelog.TimeRangeRequest.fromObject(object.timeRange);
            }
            if (object.activityPeriods) {
                if (!Array.isArray(object.activityPeriods))
                    throw TypeError(".lifelog.ActivitySummary.activityPeriods: array expected");
                message.activityPeriods = [];
                for (let i = 0; i < object.activityPeriods.length; ++i) {
                    if (typeof object.activityPeriods[i] !== "object")
                        throw TypeError(".lifelog.ActivitySummary.activityPeriods: object expected");
                    message.activityPeriods[i] = $root.lifelog.ActivityPeriod.fromObject(object.activityPeriods[i]);
                }
            }
            if (object.appUsage) {
                if (typeof object.appUsage !== "object")
                    throw TypeError(".lifelog.ActivitySummary.appUsage: object expected");
                message.appUsage = {};
                for (let keys = Object.keys(object.appUsage), i = 0; i < keys.length; ++i)
                    message.appUsage[keys[i]] = Number(object.appUsage[keys[i]]);
            }
            if (object.totalScreenshots != null)
                message.totalScreenshots = object.totalScreenshots | 0;
            if (object.totalCameraFrames != null)
                message.totalCameraFrames = object.totalCameraFrames | 0;
            if (object.totalByLogger) {
                if (typeof object.totalByLogger !== "object")
                    throw TypeError(".lifelog.ActivitySummary.totalByLogger: object expected");
                message.totalByLogger = {};
                for (let keys = Object.keys(object.totalByLogger), i = 0; i < keys.length; ++i)
                    message.totalByLogger[keys[i]] = object.totalByLogger[keys[i]] | 0;
            }
            return message;
        };

        /**
         * Creates a plain object from an ActivitySummary message. Also converts values to other types if specified.
         * @function toObject
         * @memberof lifelog.ActivitySummary
         * @static
         * @param {lifelog.ActivitySummary} message ActivitySummary
         * @param {$protobuf.IConversionOptions} [options] Conversion options
         * @returns {Object.<string,*>} Plain object
         */
        ActivitySummary.toObject = function toObject(message, options) {
            if (!options)
                options = {};
            let object = {};
            if (options.arrays || options.defaults)
                object.activityPeriods = [];
            if (options.objects || options.defaults) {
                object.appUsage = {};
                object.totalByLogger = {};
            }
            if (options.defaults) {
                object.timeRange = null;
                object.totalScreenshots = 0;
                object.totalCameraFrames = 0;
            }
            if (message.timeRange != null && message.hasOwnProperty("timeRange"))
                object.timeRange = $root.lifelog.TimeRangeRequest.toObject(message.timeRange, options);
            if (message.activityPeriods && message.activityPeriods.length) {
                object.activityPeriods = [];
                for (let j = 0; j < message.activityPeriods.length; ++j)
                    object.activityPeriods[j] = $root.lifelog.ActivityPeriod.toObject(message.activityPeriods[j], options);
            }
            let keys2;
            if (message.appUsage && (keys2 = Object.keys(message.appUsage)).length) {
                object.appUsage = {};
                for (let j = 0; j < keys2.length; ++j)
                    object.appUsage[keys2[j]] = options.json && !isFinite(message.appUsage[keys2[j]]) ? String(message.appUsage[keys2[j]]) : message.appUsage[keys2[j]];
            }
            if (message.totalScreenshots != null && message.hasOwnProperty("totalScreenshots"))
                object.totalScreenshots = message.totalScreenshots;
            if (message.totalCameraFrames != null && message.hasOwnProperty("totalCameraFrames"))
                object.totalCameraFrames = message.totalCameraFrames;
            if (message.totalByLogger && (keys2 = Object.keys(message.totalByLogger)).length) {
                object.totalByLogger = {};
                for (let j = 0; j < keys2.length; ++j)
                    object.totalByLogger[keys2[j]] = message.totalByLogger[keys2[j]];
            }
            return object;
        };

        /**
         * Converts this ActivitySummary to JSON.
         * @function toJSON
         * @memberof lifelog.ActivitySummary
         * @instance
         * @returns {Object.<string,*>} JSON object
         */
        ActivitySummary.prototype.toJSON = function toJSON() {
            return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
        };

        /**
         * Gets the default type url for ActivitySummary
         * @function getTypeUrl
         * @memberof lifelog.ActivitySummary
         * @static
         * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns {string} The default type url
         */
        ActivitySummary.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
            if (typeUrlPrefix === undefined) {
                typeUrlPrefix = "type.googleapis.com";
            }
            return typeUrlPrefix + "/lifelog.ActivitySummary";
        };

        return ActivitySummary;
    })();

    lifelog.ActivityPeriod = (function() {

        /**
         * Properties of an ActivityPeriod.
         * @memberof lifelog
         * @interface IActivityPeriod
         * @property {string|null} [startTime] ActivityPeriod startTime
         * @property {string|null} [endTime] ActivityPeriod endTime
         * @property {string|null} [primaryActivity] ActivityPeriod primaryActivity
         * @property {Object.<string,number>|null} [appsUsed] ActivityPeriod appsUsed
         * @property {number|null} [activityLevel] ActivityPeriod activityLevel
         */

        /**
         * Constructs a new ActivityPeriod.
         * @memberof lifelog
         * @classdesc Represents an ActivityPeriod.
         * @implements IActivityPeriod
         * @constructor
         * @param {lifelog.IActivityPeriod=} [properties] Properties to set
         */
        function ActivityPeriod(properties) {
            this.appsUsed = {};
            if (properties)
                for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                    if (properties[keys[i]] != null)
                        this[keys[i]] = properties[keys[i]];
        }

        /**
         * ActivityPeriod startTime.
         * @member {string} startTime
         * @memberof lifelog.ActivityPeriod
         * @instance
         */
        ActivityPeriod.prototype.startTime = "";

        /**
         * ActivityPeriod endTime.
         * @member {string} endTime
         * @memberof lifelog.ActivityPeriod
         * @instance
         */
        ActivityPeriod.prototype.endTime = "";

        /**
         * ActivityPeriod primaryActivity.
         * @member {string} primaryActivity
         * @memberof lifelog.ActivityPeriod
         * @instance
         */
        ActivityPeriod.prototype.primaryActivity = "";

        /**
         * ActivityPeriod appsUsed.
         * @member {Object.<string,number>} appsUsed
         * @memberof lifelog.ActivityPeriod
         * @instance
         */
        ActivityPeriod.prototype.appsUsed = $util.emptyObject;

        /**
         * ActivityPeriod activityLevel.
         * @member {number} activityLevel
         * @memberof lifelog.ActivityPeriod
         * @instance
         */
        ActivityPeriod.prototype.activityLevel = 0;

        /**
         * Creates a new ActivityPeriod instance using the specified properties.
         * @function create
         * @memberof lifelog.ActivityPeriod
         * @static
         * @param {lifelog.IActivityPeriod=} [properties] Properties to set
         * @returns {lifelog.ActivityPeriod} ActivityPeriod instance
         */
        ActivityPeriod.create = function create(properties) {
            return new ActivityPeriod(properties);
        };

        /**
         * Encodes the specified ActivityPeriod message. Does not implicitly {@link lifelog.ActivityPeriod.verify|verify} messages.
         * @function encode
         * @memberof lifelog.ActivityPeriod
         * @static
         * @param {lifelog.IActivityPeriod} message ActivityPeriod message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        ActivityPeriod.encode = function encode(message, writer) {
            if (!writer)
                writer = $Writer.create();
            if (message.startTime != null && Object.hasOwnProperty.call(message, "startTime"))
                writer.uint32(/* id 1, wireType 2 =*/10).string(message.startTime);
            if (message.endTime != null && Object.hasOwnProperty.call(message, "endTime"))
                writer.uint32(/* id 2, wireType 2 =*/18).string(message.endTime);
            if (message.primaryActivity != null && Object.hasOwnProperty.call(message, "primaryActivity"))
                writer.uint32(/* id 3, wireType 2 =*/26).string(message.primaryActivity);
            if (message.appsUsed != null && Object.hasOwnProperty.call(message, "appsUsed"))
                for (let keys = Object.keys(message.appsUsed), i = 0; i < keys.length; ++i)
                    writer.uint32(/* id 4, wireType 2 =*/34).fork().uint32(/* id 1, wireType 2 =*/10).string(keys[i]).uint32(/* id 2, wireType 1 =*/17).double(message.appsUsed[keys[i]]).ldelim();
            if (message.activityLevel != null && Object.hasOwnProperty.call(message, "activityLevel"))
                writer.uint32(/* id 5, wireType 1 =*/41).double(message.activityLevel);
            return writer;
        };

        /**
         * Encodes the specified ActivityPeriod message, length delimited. Does not implicitly {@link lifelog.ActivityPeriod.verify|verify} messages.
         * @function encodeDelimited
         * @memberof lifelog.ActivityPeriod
         * @static
         * @param {lifelog.IActivityPeriod} message ActivityPeriod message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        ActivityPeriod.encodeDelimited = function encodeDelimited(message, writer) {
            return this.encode(message, writer).ldelim();
        };

        /**
         * Decodes an ActivityPeriod message from the specified reader or buffer.
         * @function decode
         * @memberof lifelog.ActivityPeriod
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @param {number} [length] Message length if known beforehand
         * @returns {lifelog.ActivityPeriod} ActivityPeriod
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        ActivityPeriod.decode = function decode(reader, length, error) {
            if (!(reader instanceof $Reader))
                reader = $Reader.create(reader);
            let end = length === undefined ? reader.len : reader.pos + length, message = new $root.lifelog.ActivityPeriod(), key, value;
            while (reader.pos < end) {
                let tag = reader.uint32();
                if (tag === error)
                    break;
                switch (tag >>> 3) {
                case 1: {
                        message.startTime = reader.string();
                        break;
                    }
                case 2: {
                        message.endTime = reader.string();
                        break;
                    }
                case 3: {
                        message.primaryActivity = reader.string();
                        break;
                    }
                case 4: {
                        if (message.appsUsed === $util.emptyObject)
                            message.appsUsed = {};
                        let end2 = reader.uint32() + reader.pos;
                        key = "";
                        value = 0;
                        while (reader.pos < end2) {
                            let tag2 = reader.uint32();
                            switch (tag2 >>> 3) {
                            case 1:
                                key = reader.string();
                                break;
                            case 2:
                                value = reader.double();
                                break;
                            default:
                                reader.skipType(tag2 & 7);
                                break;
                            }
                        }
                        message.appsUsed[key] = value;
                        break;
                    }
                case 5: {
                        message.activityLevel = reader.double();
                        break;
                    }
                default:
                    reader.skipType(tag & 7);
                    break;
                }
            }
            return message;
        };

        /**
         * Decodes an ActivityPeriod message from the specified reader or buffer, length delimited.
         * @function decodeDelimited
         * @memberof lifelog.ActivityPeriod
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @returns {lifelog.ActivityPeriod} ActivityPeriod
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        ActivityPeriod.decodeDelimited = function decodeDelimited(reader) {
            if (!(reader instanceof $Reader))
                reader = new $Reader(reader);
            return this.decode(reader, reader.uint32());
        };

        /**
         * Verifies an ActivityPeriod message.
         * @function verify
         * @memberof lifelog.ActivityPeriod
         * @static
         * @param {Object.<string,*>} message Plain object to verify
         * @returns {string|null} `null` if valid, otherwise the reason why it is not
         */
        ActivityPeriod.verify = function verify(message) {
            if (typeof message !== "object" || message === null)
                return "object expected";
            if (message.startTime != null && message.hasOwnProperty("startTime"))
                if (!$util.isString(message.startTime))
                    return "startTime: string expected";
            if (message.endTime != null && message.hasOwnProperty("endTime"))
                if (!$util.isString(message.endTime))
                    return "endTime: string expected";
            if (message.primaryActivity != null && message.hasOwnProperty("primaryActivity"))
                if (!$util.isString(message.primaryActivity))
                    return "primaryActivity: string expected";
            if (message.appsUsed != null && message.hasOwnProperty("appsUsed")) {
                if (!$util.isObject(message.appsUsed))
                    return "appsUsed: object expected";
                let key = Object.keys(message.appsUsed);
                for (let i = 0; i < key.length; ++i)
                    if (typeof message.appsUsed[key[i]] !== "number")
                        return "appsUsed: number{k:string} expected";
            }
            if (message.activityLevel != null && message.hasOwnProperty("activityLevel"))
                if (typeof message.activityLevel !== "number")
                    return "activityLevel: number expected";
            return null;
        };

        /**
         * Creates an ActivityPeriod message from a plain object. Also converts values to their respective internal types.
         * @function fromObject
         * @memberof lifelog.ActivityPeriod
         * @static
         * @param {Object.<string,*>} object Plain object
         * @returns {lifelog.ActivityPeriod} ActivityPeriod
         */
        ActivityPeriod.fromObject = function fromObject(object) {
            if (object instanceof $root.lifelog.ActivityPeriod)
                return object;
            let message = new $root.lifelog.ActivityPeriod();
            if (object.startTime != null)
                message.startTime = String(object.startTime);
            if (object.endTime != null)
                message.endTime = String(object.endTime);
            if (object.primaryActivity != null)
                message.primaryActivity = String(object.primaryActivity);
            if (object.appsUsed) {
                if (typeof object.appsUsed !== "object")
                    throw TypeError(".lifelog.ActivityPeriod.appsUsed: object expected");
                message.appsUsed = {};
                for (let keys = Object.keys(object.appsUsed), i = 0; i < keys.length; ++i)
                    message.appsUsed[keys[i]] = Number(object.appsUsed[keys[i]]);
            }
            if (object.activityLevel != null)
                message.activityLevel = Number(object.activityLevel);
            return message;
        };

        /**
         * Creates a plain object from an ActivityPeriod message. Also converts values to other types if specified.
         * @function toObject
         * @memberof lifelog.ActivityPeriod
         * @static
         * @param {lifelog.ActivityPeriod} message ActivityPeriod
         * @param {$protobuf.IConversionOptions} [options] Conversion options
         * @returns {Object.<string,*>} Plain object
         */
        ActivityPeriod.toObject = function toObject(message, options) {
            if (!options)
                options = {};
            let object = {};
            if (options.objects || options.defaults)
                object.appsUsed = {};
            if (options.defaults) {
                object.startTime = "";
                object.endTime = "";
                object.primaryActivity = "";
                object.activityLevel = 0;
            }
            if (message.startTime != null && message.hasOwnProperty("startTime"))
                object.startTime = message.startTime;
            if (message.endTime != null && message.hasOwnProperty("endTime"))
                object.endTime = message.endTime;
            if (message.primaryActivity != null && message.hasOwnProperty("primaryActivity"))
                object.primaryActivity = message.primaryActivity;
            let keys2;
            if (message.appsUsed && (keys2 = Object.keys(message.appsUsed)).length) {
                object.appsUsed = {};
                for (let j = 0; j < keys2.length; ++j)
                    object.appsUsed[keys2[j]] = options.json && !isFinite(message.appsUsed[keys2[j]]) ? String(message.appsUsed[keys2[j]]) : message.appsUsed[keys2[j]];
            }
            if (message.activityLevel != null && message.hasOwnProperty("activityLevel"))
                object.activityLevel = options.json && !isFinite(message.activityLevel) ? String(message.activityLevel) : message.activityLevel;
            return object;
        };

        /**
         * Converts this ActivityPeriod to JSON.
         * @function toJSON
         * @memberof lifelog.ActivityPeriod
         * @instance
         * @returns {Object.<string,*>} JSON object
         */
        ActivityPeriod.prototype.toJSON = function toJSON() {
            return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
        };

        /**
         * Gets the default type url for ActivityPeriod
         * @function getTypeUrl
         * @memberof lifelog.ActivityPeriod
         * @static
         * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns {string} The default type url
         */
        ActivityPeriod.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
            if (typeUrlPrefix === undefined) {
                typeUrlPrefix = "type.googleapis.com";
            }
            return typeUrlPrefix + "/lifelog.ActivityPeriod";
        };

        return ActivityPeriod;
    })();

    lifelog.LoginRequest = (function() {

        /**
         * Properties of a LoginRequest.
         * @memberof lifelog
         * @interface ILoginRequest
         * @property {string|null} [username] LoginRequest username
         * @property {string|null} [password] LoginRequest password
         */

        /**
         * Constructs a new LoginRequest.
         * @memberof lifelog
         * @classdesc Represents a LoginRequest.
         * @implements ILoginRequest
         * @constructor
         * @param {lifelog.ILoginRequest=} [properties] Properties to set
         */
        function LoginRequest(properties) {
            if (properties)
                for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                    if (properties[keys[i]] != null)
                        this[keys[i]] = properties[keys[i]];
        }

        /**
         * LoginRequest username.
         * @member {string} username
         * @memberof lifelog.LoginRequest
         * @instance
         */
        LoginRequest.prototype.username = "";

        /**
         * LoginRequest password.
         * @member {string} password
         * @memberof lifelog.LoginRequest
         * @instance
         */
        LoginRequest.prototype.password = "";

        /**
         * Creates a new LoginRequest instance using the specified properties.
         * @function create
         * @memberof lifelog.LoginRequest
         * @static
         * @param {lifelog.ILoginRequest=} [properties] Properties to set
         * @returns {lifelog.LoginRequest} LoginRequest instance
         */
        LoginRequest.create = function create(properties) {
            return new LoginRequest(properties);
        };

        /**
         * Encodes the specified LoginRequest message. Does not implicitly {@link lifelog.LoginRequest.verify|verify} messages.
         * @function encode
         * @memberof lifelog.LoginRequest
         * @static
         * @param {lifelog.ILoginRequest} message LoginRequest message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        LoginRequest.encode = function encode(message, writer) {
            if (!writer)
                writer = $Writer.create();
            if (message.username != null && Object.hasOwnProperty.call(message, "username"))
                writer.uint32(/* id 1, wireType 2 =*/10).string(message.username);
            if (message.password != null && Object.hasOwnProperty.call(message, "password"))
                writer.uint32(/* id 2, wireType 2 =*/18).string(message.password);
            return writer;
        };

        /**
         * Encodes the specified LoginRequest message, length delimited. Does not implicitly {@link lifelog.LoginRequest.verify|verify} messages.
         * @function encodeDelimited
         * @memberof lifelog.LoginRequest
         * @static
         * @param {lifelog.ILoginRequest} message LoginRequest message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        LoginRequest.encodeDelimited = function encodeDelimited(message, writer) {
            return this.encode(message, writer).ldelim();
        };

        /**
         * Decodes a LoginRequest message from the specified reader or buffer.
         * @function decode
         * @memberof lifelog.LoginRequest
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @param {number} [length] Message length if known beforehand
         * @returns {lifelog.LoginRequest} LoginRequest
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        LoginRequest.decode = function decode(reader, length, error) {
            if (!(reader instanceof $Reader))
                reader = $Reader.create(reader);
            let end = length === undefined ? reader.len : reader.pos + length, message = new $root.lifelog.LoginRequest();
            while (reader.pos < end) {
                let tag = reader.uint32();
                if (tag === error)
                    break;
                switch (tag >>> 3) {
                case 1: {
                        message.username = reader.string();
                        break;
                    }
                case 2: {
                        message.password = reader.string();
                        break;
                    }
                default:
                    reader.skipType(tag & 7);
                    break;
                }
            }
            return message;
        };

        /**
         * Decodes a LoginRequest message from the specified reader or buffer, length delimited.
         * @function decodeDelimited
         * @memberof lifelog.LoginRequest
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @returns {lifelog.LoginRequest} LoginRequest
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        LoginRequest.decodeDelimited = function decodeDelimited(reader) {
            if (!(reader instanceof $Reader))
                reader = new $Reader(reader);
            return this.decode(reader, reader.uint32());
        };

        /**
         * Verifies a LoginRequest message.
         * @function verify
         * @memberof lifelog.LoginRequest
         * @static
         * @param {Object.<string,*>} message Plain object to verify
         * @returns {string|null} `null` if valid, otherwise the reason why it is not
         */
        LoginRequest.verify = function verify(message) {
            if (typeof message !== "object" || message === null)
                return "object expected";
            if (message.username != null && message.hasOwnProperty("username"))
                if (!$util.isString(message.username))
                    return "username: string expected";
            if (message.password != null && message.hasOwnProperty("password"))
                if (!$util.isString(message.password))
                    return "password: string expected";
            return null;
        };

        /**
         * Creates a LoginRequest message from a plain object. Also converts values to their respective internal types.
         * @function fromObject
         * @memberof lifelog.LoginRequest
         * @static
         * @param {Object.<string,*>} object Plain object
         * @returns {lifelog.LoginRequest} LoginRequest
         */
        LoginRequest.fromObject = function fromObject(object) {
            if (object instanceof $root.lifelog.LoginRequest)
                return object;
            let message = new $root.lifelog.LoginRequest();
            if (object.username != null)
                message.username = String(object.username);
            if (object.password != null)
                message.password = String(object.password);
            return message;
        };

        /**
         * Creates a plain object from a LoginRequest message. Also converts values to other types if specified.
         * @function toObject
         * @memberof lifelog.LoginRequest
         * @static
         * @param {lifelog.LoginRequest} message LoginRequest
         * @param {$protobuf.IConversionOptions} [options] Conversion options
         * @returns {Object.<string,*>} Plain object
         */
        LoginRequest.toObject = function toObject(message, options) {
            if (!options)
                options = {};
            let object = {};
            if (options.defaults) {
                object.username = "";
                object.password = "";
            }
            if (message.username != null && message.hasOwnProperty("username"))
                object.username = message.username;
            if (message.password != null && message.hasOwnProperty("password"))
                object.password = message.password;
            return object;
        };

        /**
         * Converts this LoginRequest to JSON.
         * @function toJSON
         * @memberof lifelog.LoginRequest
         * @instance
         * @returns {Object.<string,*>} JSON object
         */
        LoginRequest.prototype.toJSON = function toJSON() {
            return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
        };

        /**
         * Gets the default type url for LoginRequest
         * @function getTypeUrl
         * @memberof lifelog.LoginRequest
         * @static
         * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns {string} The default type url
         */
        LoginRequest.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
            if (typeUrlPrefix === undefined) {
                typeUrlPrefix = "type.googleapis.com";
            }
            return typeUrlPrefix + "/lifelog.LoginRequest";
        };

        return LoginRequest;
    })();

    lifelog.LoginResponse = (function() {

        /**
         * Properties of a LoginResponse.
         * @memberof lifelog
         * @interface ILoginResponse
         * @property {string|null} [token] LoginResponse token
         * @property {boolean|null} [success] LoginResponse success
         * @property {string|null} [errorMessage] LoginResponse errorMessage
         * @property {lifelog.IUserProfile|null} [userProfile] LoginResponse userProfile
         */

        /**
         * Constructs a new LoginResponse.
         * @memberof lifelog
         * @classdesc Represents a LoginResponse.
         * @implements ILoginResponse
         * @constructor
         * @param {lifelog.ILoginResponse=} [properties] Properties to set
         */
        function LoginResponse(properties) {
            if (properties)
                for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                    if (properties[keys[i]] != null)
                        this[keys[i]] = properties[keys[i]];
        }

        /**
         * LoginResponse token.
         * @member {string} token
         * @memberof lifelog.LoginResponse
         * @instance
         */
        LoginResponse.prototype.token = "";

        /**
         * LoginResponse success.
         * @member {boolean} success
         * @memberof lifelog.LoginResponse
         * @instance
         */
        LoginResponse.prototype.success = false;

        /**
         * LoginResponse errorMessage.
         * @member {string} errorMessage
         * @memberof lifelog.LoginResponse
         * @instance
         */
        LoginResponse.prototype.errorMessage = "";

        /**
         * LoginResponse userProfile.
         * @member {lifelog.IUserProfile|null|undefined} userProfile
         * @memberof lifelog.LoginResponse
         * @instance
         */
        LoginResponse.prototype.userProfile = null;

        /**
         * Creates a new LoginResponse instance using the specified properties.
         * @function create
         * @memberof lifelog.LoginResponse
         * @static
         * @param {lifelog.ILoginResponse=} [properties] Properties to set
         * @returns {lifelog.LoginResponse} LoginResponse instance
         */
        LoginResponse.create = function create(properties) {
            return new LoginResponse(properties);
        };

        /**
         * Encodes the specified LoginResponse message. Does not implicitly {@link lifelog.LoginResponse.verify|verify} messages.
         * @function encode
         * @memberof lifelog.LoginResponse
         * @static
         * @param {lifelog.ILoginResponse} message LoginResponse message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        LoginResponse.encode = function encode(message, writer) {
            if (!writer)
                writer = $Writer.create();
            if (message.token != null && Object.hasOwnProperty.call(message, "token"))
                writer.uint32(/* id 1, wireType 2 =*/10).string(message.token);
            if (message.success != null && Object.hasOwnProperty.call(message, "success"))
                writer.uint32(/* id 2, wireType 0 =*/16).bool(message.success);
            if (message.errorMessage != null && Object.hasOwnProperty.call(message, "errorMessage"))
                writer.uint32(/* id 3, wireType 2 =*/26).string(message.errorMessage);
            if (message.userProfile != null && Object.hasOwnProperty.call(message, "userProfile"))
                $root.lifelog.UserProfile.encode(message.userProfile, writer.uint32(/* id 4, wireType 2 =*/34).fork()).ldelim();
            return writer;
        };

        /**
         * Encodes the specified LoginResponse message, length delimited. Does not implicitly {@link lifelog.LoginResponse.verify|verify} messages.
         * @function encodeDelimited
         * @memberof lifelog.LoginResponse
         * @static
         * @param {lifelog.ILoginResponse} message LoginResponse message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        LoginResponse.encodeDelimited = function encodeDelimited(message, writer) {
            return this.encode(message, writer).ldelim();
        };

        /**
         * Decodes a LoginResponse message from the specified reader or buffer.
         * @function decode
         * @memberof lifelog.LoginResponse
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @param {number} [length] Message length if known beforehand
         * @returns {lifelog.LoginResponse} LoginResponse
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        LoginResponse.decode = function decode(reader, length, error) {
            if (!(reader instanceof $Reader))
                reader = $Reader.create(reader);
            let end = length === undefined ? reader.len : reader.pos + length, message = new $root.lifelog.LoginResponse();
            while (reader.pos < end) {
                let tag = reader.uint32();
                if (tag === error)
                    break;
                switch (tag >>> 3) {
                case 1: {
                        message.token = reader.string();
                        break;
                    }
                case 2: {
                        message.success = reader.bool();
                        break;
                    }
                case 3: {
                        message.errorMessage = reader.string();
                        break;
                    }
                case 4: {
                        message.userProfile = $root.lifelog.UserProfile.decode(reader, reader.uint32());
                        break;
                    }
                default:
                    reader.skipType(tag & 7);
                    break;
                }
            }
            return message;
        };

        /**
         * Decodes a LoginResponse message from the specified reader or buffer, length delimited.
         * @function decodeDelimited
         * @memberof lifelog.LoginResponse
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @returns {lifelog.LoginResponse} LoginResponse
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        LoginResponse.decodeDelimited = function decodeDelimited(reader) {
            if (!(reader instanceof $Reader))
                reader = new $Reader(reader);
            return this.decode(reader, reader.uint32());
        };

        /**
         * Verifies a LoginResponse message.
         * @function verify
         * @memberof lifelog.LoginResponse
         * @static
         * @param {Object.<string,*>} message Plain object to verify
         * @returns {string|null} `null` if valid, otherwise the reason why it is not
         */
        LoginResponse.verify = function verify(message) {
            if (typeof message !== "object" || message === null)
                return "object expected";
            if (message.token != null && message.hasOwnProperty("token"))
                if (!$util.isString(message.token))
                    return "token: string expected";
            if (message.success != null && message.hasOwnProperty("success"))
                if (typeof message.success !== "boolean")
                    return "success: boolean expected";
            if (message.errorMessage != null && message.hasOwnProperty("errorMessage"))
                if (!$util.isString(message.errorMessage))
                    return "errorMessage: string expected";
            if (message.userProfile != null && message.hasOwnProperty("userProfile")) {
                let error = $root.lifelog.UserProfile.verify(message.userProfile);
                if (error)
                    return "userProfile." + error;
            }
            return null;
        };

        /**
         * Creates a LoginResponse message from a plain object. Also converts values to their respective internal types.
         * @function fromObject
         * @memberof lifelog.LoginResponse
         * @static
         * @param {Object.<string,*>} object Plain object
         * @returns {lifelog.LoginResponse} LoginResponse
         */
        LoginResponse.fromObject = function fromObject(object) {
            if (object instanceof $root.lifelog.LoginResponse)
                return object;
            let message = new $root.lifelog.LoginResponse();
            if (object.token != null)
                message.token = String(object.token);
            if (object.success != null)
                message.success = Boolean(object.success);
            if (object.errorMessage != null)
                message.errorMessage = String(object.errorMessage);
            if (object.userProfile != null) {
                if (typeof object.userProfile !== "object")
                    throw TypeError(".lifelog.LoginResponse.userProfile: object expected");
                message.userProfile = $root.lifelog.UserProfile.fromObject(object.userProfile);
            }
            return message;
        };

        /**
         * Creates a plain object from a LoginResponse message. Also converts values to other types if specified.
         * @function toObject
         * @memberof lifelog.LoginResponse
         * @static
         * @param {lifelog.LoginResponse} message LoginResponse
         * @param {$protobuf.IConversionOptions} [options] Conversion options
         * @returns {Object.<string,*>} Plain object
         */
        LoginResponse.toObject = function toObject(message, options) {
            if (!options)
                options = {};
            let object = {};
            if (options.defaults) {
                object.token = "";
                object.success = false;
                object.errorMessage = "";
                object.userProfile = null;
            }
            if (message.token != null && message.hasOwnProperty("token"))
                object.token = message.token;
            if (message.success != null && message.hasOwnProperty("success"))
                object.success = message.success;
            if (message.errorMessage != null && message.hasOwnProperty("errorMessage"))
                object.errorMessage = message.errorMessage;
            if (message.userProfile != null && message.hasOwnProperty("userProfile"))
                object.userProfile = $root.lifelog.UserProfile.toObject(message.userProfile, options);
            return object;
        };

        /**
         * Converts this LoginResponse to JSON.
         * @function toJSON
         * @memberof lifelog.LoginResponse
         * @instance
         * @returns {Object.<string,*>} JSON object
         */
        LoginResponse.prototype.toJSON = function toJSON() {
            return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
        };

        /**
         * Gets the default type url for LoginResponse
         * @function getTypeUrl
         * @memberof lifelog.LoginResponse
         * @static
         * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns {string} The default type url
         */
        LoginResponse.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
            if (typeUrlPrefix === undefined) {
                typeUrlPrefix = "type.googleapis.com";
            }
            return typeUrlPrefix + "/lifelog.LoginResponse";
        };

        return LoginResponse;
    })();

    lifelog.RegisterRequest = (function() {

        /**
         * Properties of a RegisterRequest.
         * @memberof lifelog
         * @interface IRegisterRequest
         * @property {string|null} [username] RegisterRequest username
         * @property {string|null} [password] RegisterRequest password
         * @property {string|null} [email] RegisterRequest email
         * @property {string|null} [displayName] RegisterRequest displayName
         */

        /**
         * Constructs a new RegisterRequest.
         * @memberof lifelog
         * @classdesc Represents a RegisterRequest.
         * @implements IRegisterRequest
         * @constructor
         * @param {lifelog.IRegisterRequest=} [properties] Properties to set
         */
        function RegisterRequest(properties) {
            if (properties)
                for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                    if (properties[keys[i]] != null)
                        this[keys[i]] = properties[keys[i]];
        }

        /**
         * RegisterRequest username.
         * @member {string} username
         * @memberof lifelog.RegisterRequest
         * @instance
         */
        RegisterRequest.prototype.username = "";

        /**
         * RegisterRequest password.
         * @member {string} password
         * @memberof lifelog.RegisterRequest
         * @instance
         */
        RegisterRequest.prototype.password = "";

        /**
         * RegisterRequest email.
         * @member {string} email
         * @memberof lifelog.RegisterRequest
         * @instance
         */
        RegisterRequest.prototype.email = "";

        /**
         * RegisterRequest displayName.
         * @member {string} displayName
         * @memberof lifelog.RegisterRequest
         * @instance
         */
        RegisterRequest.prototype.displayName = "";

        /**
         * Creates a new RegisterRequest instance using the specified properties.
         * @function create
         * @memberof lifelog.RegisterRequest
         * @static
         * @param {lifelog.IRegisterRequest=} [properties] Properties to set
         * @returns {lifelog.RegisterRequest} RegisterRequest instance
         */
        RegisterRequest.create = function create(properties) {
            return new RegisterRequest(properties);
        };

        /**
         * Encodes the specified RegisterRequest message. Does not implicitly {@link lifelog.RegisterRequest.verify|verify} messages.
         * @function encode
         * @memberof lifelog.RegisterRequest
         * @static
         * @param {lifelog.IRegisterRequest} message RegisterRequest message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        RegisterRequest.encode = function encode(message, writer) {
            if (!writer)
                writer = $Writer.create();
            if (message.username != null && Object.hasOwnProperty.call(message, "username"))
                writer.uint32(/* id 1, wireType 2 =*/10).string(message.username);
            if (message.password != null && Object.hasOwnProperty.call(message, "password"))
                writer.uint32(/* id 2, wireType 2 =*/18).string(message.password);
            if (message.email != null && Object.hasOwnProperty.call(message, "email"))
                writer.uint32(/* id 3, wireType 2 =*/26).string(message.email);
            if (message.displayName != null && Object.hasOwnProperty.call(message, "displayName"))
                writer.uint32(/* id 4, wireType 2 =*/34).string(message.displayName);
            return writer;
        };

        /**
         * Encodes the specified RegisterRequest message, length delimited. Does not implicitly {@link lifelog.RegisterRequest.verify|verify} messages.
         * @function encodeDelimited
         * @memberof lifelog.RegisterRequest
         * @static
         * @param {lifelog.IRegisterRequest} message RegisterRequest message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        RegisterRequest.encodeDelimited = function encodeDelimited(message, writer) {
            return this.encode(message, writer).ldelim();
        };

        /**
         * Decodes a RegisterRequest message from the specified reader or buffer.
         * @function decode
         * @memberof lifelog.RegisterRequest
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @param {number} [length] Message length if known beforehand
         * @returns {lifelog.RegisterRequest} RegisterRequest
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        RegisterRequest.decode = function decode(reader, length, error) {
            if (!(reader instanceof $Reader))
                reader = $Reader.create(reader);
            let end = length === undefined ? reader.len : reader.pos + length, message = new $root.lifelog.RegisterRequest();
            while (reader.pos < end) {
                let tag = reader.uint32();
                if (tag === error)
                    break;
                switch (tag >>> 3) {
                case 1: {
                        message.username = reader.string();
                        break;
                    }
                case 2: {
                        message.password = reader.string();
                        break;
                    }
                case 3: {
                        message.email = reader.string();
                        break;
                    }
                case 4: {
                        message.displayName = reader.string();
                        break;
                    }
                default:
                    reader.skipType(tag & 7);
                    break;
                }
            }
            return message;
        };

        /**
         * Decodes a RegisterRequest message from the specified reader or buffer, length delimited.
         * @function decodeDelimited
         * @memberof lifelog.RegisterRequest
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @returns {lifelog.RegisterRequest} RegisterRequest
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        RegisterRequest.decodeDelimited = function decodeDelimited(reader) {
            if (!(reader instanceof $Reader))
                reader = new $Reader(reader);
            return this.decode(reader, reader.uint32());
        };

        /**
         * Verifies a RegisterRequest message.
         * @function verify
         * @memberof lifelog.RegisterRequest
         * @static
         * @param {Object.<string,*>} message Plain object to verify
         * @returns {string|null} `null` if valid, otherwise the reason why it is not
         */
        RegisterRequest.verify = function verify(message) {
            if (typeof message !== "object" || message === null)
                return "object expected";
            if (message.username != null && message.hasOwnProperty("username"))
                if (!$util.isString(message.username))
                    return "username: string expected";
            if (message.password != null && message.hasOwnProperty("password"))
                if (!$util.isString(message.password))
                    return "password: string expected";
            if (message.email != null && message.hasOwnProperty("email"))
                if (!$util.isString(message.email))
                    return "email: string expected";
            if (message.displayName != null && message.hasOwnProperty("displayName"))
                if (!$util.isString(message.displayName))
                    return "displayName: string expected";
            return null;
        };

        /**
         * Creates a RegisterRequest message from a plain object. Also converts values to their respective internal types.
         * @function fromObject
         * @memberof lifelog.RegisterRequest
         * @static
         * @param {Object.<string,*>} object Plain object
         * @returns {lifelog.RegisterRequest} RegisterRequest
         */
        RegisterRequest.fromObject = function fromObject(object) {
            if (object instanceof $root.lifelog.RegisterRequest)
                return object;
            let message = new $root.lifelog.RegisterRequest();
            if (object.username != null)
                message.username = String(object.username);
            if (object.password != null)
                message.password = String(object.password);
            if (object.email != null)
                message.email = String(object.email);
            if (object.displayName != null)
                message.displayName = String(object.displayName);
            return message;
        };

        /**
         * Creates a plain object from a RegisterRequest message. Also converts values to other types if specified.
         * @function toObject
         * @memberof lifelog.RegisterRequest
         * @static
         * @param {lifelog.RegisterRequest} message RegisterRequest
         * @param {$protobuf.IConversionOptions} [options] Conversion options
         * @returns {Object.<string,*>} Plain object
         */
        RegisterRequest.toObject = function toObject(message, options) {
            if (!options)
                options = {};
            let object = {};
            if (options.defaults) {
                object.username = "";
                object.password = "";
                object.email = "";
                object.displayName = "";
            }
            if (message.username != null && message.hasOwnProperty("username"))
                object.username = message.username;
            if (message.password != null && message.hasOwnProperty("password"))
                object.password = message.password;
            if (message.email != null && message.hasOwnProperty("email"))
                object.email = message.email;
            if (message.displayName != null && message.hasOwnProperty("displayName"))
                object.displayName = message.displayName;
            return object;
        };

        /**
         * Converts this RegisterRequest to JSON.
         * @function toJSON
         * @memberof lifelog.RegisterRequest
         * @instance
         * @returns {Object.<string,*>} JSON object
         */
        RegisterRequest.prototype.toJSON = function toJSON() {
            return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
        };

        /**
         * Gets the default type url for RegisterRequest
         * @function getTypeUrl
         * @memberof lifelog.RegisterRequest
         * @static
         * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns {string} The default type url
         */
        RegisterRequest.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
            if (typeUrlPrefix === undefined) {
                typeUrlPrefix = "type.googleapis.com";
            }
            return typeUrlPrefix + "/lifelog.RegisterRequest";
        };

        return RegisterRequest;
    })();

    lifelog.RegisterResponse = (function() {

        /**
         * Properties of a RegisterResponse.
         * @memberof lifelog
         * @interface IRegisterResponse
         * @property {boolean|null} [success] RegisterResponse success
         * @property {string|null} [errorMessage] RegisterResponse errorMessage
         * @property {string|null} [token] RegisterResponse token
         */

        /**
         * Constructs a new RegisterResponse.
         * @memberof lifelog
         * @classdesc Represents a RegisterResponse.
         * @implements IRegisterResponse
         * @constructor
         * @param {lifelog.IRegisterResponse=} [properties] Properties to set
         */
        function RegisterResponse(properties) {
            if (properties)
                for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                    if (properties[keys[i]] != null)
                        this[keys[i]] = properties[keys[i]];
        }

        /**
         * RegisterResponse success.
         * @member {boolean} success
         * @memberof lifelog.RegisterResponse
         * @instance
         */
        RegisterResponse.prototype.success = false;

        /**
         * RegisterResponse errorMessage.
         * @member {string} errorMessage
         * @memberof lifelog.RegisterResponse
         * @instance
         */
        RegisterResponse.prototype.errorMessage = "";

        /**
         * RegisterResponse token.
         * @member {string} token
         * @memberof lifelog.RegisterResponse
         * @instance
         */
        RegisterResponse.prototype.token = "";

        /**
         * Creates a new RegisterResponse instance using the specified properties.
         * @function create
         * @memberof lifelog.RegisterResponse
         * @static
         * @param {lifelog.IRegisterResponse=} [properties] Properties to set
         * @returns {lifelog.RegisterResponse} RegisterResponse instance
         */
        RegisterResponse.create = function create(properties) {
            return new RegisterResponse(properties);
        };

        /**
         * Encodes the specified RegisterResponse message. Does not implicitly {@link lifelog.RegisterResponse.verify|verify} messages.
         * @function encode
         * @memberof lifelog.RegisterResponse
         * @static
         * @param {lifelog.IRegisterResponse} message RegisterResponse message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        RegisterResponse.encode = function encode(message, writer) {
            if (!writer)
                writer = $Writer.create();
            if (message.success != null && Object.hasOwnProperty.call(message, "success"))
                writer.uint32(/* id 1, wireType 0 =*/8).bool(message.success);
            if (message.errorMessage != null && Object.hasOwnProperty.call(message, "errorMessage"))
                writer.uint32(/* id 2, wireType 2 =*/18).string(message.errorMessage);
            if (message.token != null && Object.hasOwnProperty.call(message, "token"))
                writer.uint32(/* id 3, wireType 2 =*/26).string(message.token);
            return writer;
        };

        /**
         * Encodes the specified RegisterResponse message, length delimited. Does not implicitly {@link lifelog.RegisterResponse.verify|verify} messages.
         * @function encodeDelimited
         * @memberof lifelog.RegisterResponse
         * @static
         * @param {lifelog.IRegisterResponse} message RegisterResponse message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        RegisterResponse.encodeDelimited = function encodeDelimited(message, writer) {
            return this.encode(message, writer).ldelim();
        };

        /**
         * Decodes a RegisterResponse message from the specified reader or buffer.
         * @function decode
         * @memberof lifelog.RegisterResponse
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @param {number} [length] Message length if known beforehand
         * @returns {lifelog.RegisterResponse} RegisterResponse
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        RegisterResponse.decode = function decode(reader, length, error) {
            if (!(reader instanceof $Reader))
                reader = $Reader.create(reader);
            let end = length === undefined ? reader.len : reader.pos + length, message = new $root.lifelog.RegisterResponse();
            while (reader.pos < end) {
                let tag = reader.uint32();
                if (tag === error)
                    break;
                switch (tag >>> 3) {
                case 1: {
                        message.success = reader.bool();
                        break;
                    }
                case 2: {
                        message.errorMessage = reader.string();
                        break;
                    }
                case 3: {
                        message.token = reader.string();
                        break;
                    }
                default:
                    reader.skipType(tag & 7);
                    break;
                }
            }
            return message;
        };

        /**
         * Decodes a RegisterResponse message from the specified reader or buffer, length delimited.
         * @function decodeDelimited
         * @memberof lifelog.RegisterResponse
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @returns {lifelog.RegisterResponse} RegisterResponse
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        RegisterResponse.decodeDelimited = function decodeDelimited(reader) {
            if (!(reader instanceof $Reader))
                reader = new $Reader(reader);
            return this.decode(reader, reader.uint32());
        };

        /**
         * Verifies a RegisterResponse message.
         * @function verify
         * @memberof lifelog.RegisterResponse
         * @static
         * @param {Object.<string,*>} message Plain object to verify
         * @returns {string|null} `null` if valid, otherwise the reason why it is not
         */
        RegisterResponse.verify = function verify(message) {
            if (typeof message !== "object" || message === null)
                return "object expected";
            if (message.success != null && message.hasOwnProperty("success"))
                if (typeof message.success !== "boolean")
                    return "success: boolean expected";
            if (message.errorMessage != null && message.hasOwnProperty("errorMessage"))
                if (!$util.isString(message.errorMessage))
                    return "errorMessage: string expected";
            if (message.token != null && message.hasOwnProperty("token"))
                if (!$util.isString(message.token))
                    return "token: string expected";
            return null;
        };

        /**
         * Creates a RegisterResponse message from a plain object. Also converts values to their respective internal types.
         * @function fromObject
         * @memberof lifelog.RegisterResponse
         * @static
         * @param {Object.<string,*>} object Plain object
         * @returns {lifelog.RegisterResponse} RegisterResponse
         */
        RegisterResponse.fromObject = function fromObject(object) {
            if (object instanceof $root.lifelog.RegisterResponse)
                return object;
            let message = new $root.lifelog.RegisterResponse();
            if (object.success != null)
                message.success = Boolean(object.success);
            if (object.errorMessage != null)
                message.errorMessage = String(object.errorMessage);
            if (object.token != null)
                message.token = String(object.token);
            return message;
        };

        /**
         * Creates a plain object from a RegisterResponse message. Also converts values to other types if specified.
         * @function toObject
         * @memberof lifelog.RegisterResponse
         * @static
         * @param {lifelog.RegisterResponse} message RegisterResponse
         * @param {$protobuf.IConversionOptions} [options] Conversion options
         * @returns {Object.<string,*>} Plain object
         */
        RegisterResponse.toObject = function toObject(message, options) {
            if (!options)
                options = {};
            let object = {};
            if (options.defaults) {
                object.success = false;
                object.errorMessage = "";
                object.token = "";
            }
            if (message.success != null && message.hasOwnProperty("success"))
                object.success = message.success;
            if (message.errorMessage != null && message.hasOwnProperty("errorMessage"))
                object.errorMessage = message.errorMessage;
            if (message.token != null && message.hasOwnProperty("token"))
                object.token = message.token;
            return object;
        };

        /**
         * Converts this RegisterResponse to JSON.
         * @function toJSON
         * @memberof lifelog.RegisterResponse
         * @instance
         * @returns {Object.<string,*>} JSON object
         */
        RegisterResponse.prototype.toJSON = function toJSON() {
            return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
        };

        /**
         * Gets the default type url for RegisterResponse
         * @function getTypeUrl
         * @memberof lifelog.RegisterResponse
         * @static
         * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns {string} The default type url
         */
        RegisterResponse.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
            if (typeUrlPrefix === undefined) {
                typeUrlPrefix = "type.googleapis.com";
            }
            return typeUrlPrefix + "/lifelog.RegisterResponse";
        };

        return RegisterResponse;
    })();

    lifelog.UserRequest = (function() {

        /**
         * Properties of a UserRequest.
         * @memberof lifelog
         * @interface IUserRequest
         * @property {string|null} [userId] UserRequest userId
         */

        /**
         * Constructs a new UserRequest.
         * @memberof lifelog
         * @classdesc Represents a UserRequest.
         * @implements IUserRequest
         * @constructor
         * @param {lifelog.IUserRequest=} [properties] Properties to set
         */
        function UserRequest(properties) {
            if (properties)
                for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                    if (properties[keys[i]] != null)
                        this[keys[i]] = properties[keys[i]];
        }

        /**
         * UserRequest userId.
         * @member {string} userId
         * @memberof lifelog.UserRequest
         * @instance
         */
        UserRequest.prototype.userId = "";

        /**
         * Creates a new UserRequest instance using the specified properties.
         * @function create
         * @memberof lifelog.UserRequest
         * @static
         * @param {lifelog.IUserRequest=} [properties] Properties to set
         * @returns {lifelog.UserRequest} UserRequest instance
         */
        UserRequest.create = function create(properties) {
            return new UserRequest(properties);
        };

        /**
         * Encodes the specified UserRequest message. Does not implicitly {@link lifelog.UserRequest.verify|verify} messages.
         * @function encode
         * @memberof lifelog.UserRequest
         * @static
         * @param {lifelog.IUserRequest} message UserRequest message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        UserRequest.encode = function encode(message, writer) {
            if (!writer)
                writer = $Writer.create();
            if (message.userId != null && Object.hasOwnProperty.call(message, "userId"))
                writer.uint32(/* id 1, wireType 2 =*/10).string(message.userId);
            return writer;
        };

        /**
         * Encodes the specified UserRequest message, length delimited. Does not implicitly {@link lifelog.UserRequest.verify|verify} messages.
         * @function encodeDelimited
         * @memberof lifelog.UserRequest
         * @static
         * @param {lifelog.IUserRequest} message UserRequest message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        UserRequest.encodeDelimited = function encodeDelimited(message, writer) {
            return this.encode(message, writer).ldelim();
        };

        /**
         * Decodes a UserRequest message from the specified reader or buffer.
         * @function decode
         * @memberof lifelog.UserRequest
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @param {number} [length] Message length if known beforehand
         * @returns {lifelog.UserRequest} UserRequest
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        UserRequest.decode = function decode(reader, length, error) {
            if (!(reader instanceof $Reader))
                reader = $Reader.create(reader);
            let end = length === undefined ? reader.len : reader.pos + length, message = new $root.lifelog.UserRequest();
            while (reader.pos < end) {
                let tag = reader.uint32();
                if (tag === error)
                    break;
                switch (tag >>> 3) {
                case 1: {
                        message.userId = reader.string();
                        break;
                    }
                default:
                    reader.skipType(tag & 7);
                    break;
                }
            }
            return message;
        };

        /**
         * Decodes a UserRequest message from the specified reader or buffer, length delimited.
         * @function decodeDelimited
         * @memberof lifelog.UserRequest
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @returns {lifelog.UserRequest} UserRequest
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        UserRequest.decodeDelimited = function decodeDelimited(reader) {
            if (!(reader instanceof $Reader))
                reader = new $Reader(reader);
            return this.decode(reader, reader.uint32());
        };

        /**
         * Verifies a UserRequest message.
         * @function verify
         * @memberof lifelog.UserRequest
         * @static
         * @param {Object.<string,*>} message Plain object to verify
         * @returns {string|null} `null` if valid, otherwise the reason why it is not
         */
        UserRequest.verify = function verify(message) {
            if (typeof message !== "object" || message === null)
                return "object expected";
            if (message.userId != null && message.hasOwnProperty("userId"))
                if (!$util.isString(message.userId))
                    return "userId: string expected";
            return null;
        };

        /**
         * Creates a UserRequest message from a plain object. Also converts values to their respective internal types.
         * @function fromObject
         * @memberof lifelog.UserRequest
         * @static
         * @param {Object.<string,*>} object Plain object
         * @returns {lifelog.UserRequest} UserRequest
         */
        UserRequest.fromObject = function fromObject(object) {
            if (object instanceof $root.lifelog.UserRequest)
                return object;
            let message = new $root.lifelog.UserRequest();
            if (object.userId != null)
                message.userId = String(object.userId);
            return message;
        };

        /**
         * Creates a plain object from a UserRequest message. Also converts values to other types if specified.
         * @function toObject
         * @memberof lifelog.UserRequest
         * @static
         * @param {lifelog.UserRequest} message UserRequest
         * @param {$protobuf.IConversionOptions} [options] Conversion options
         * @returns {Object.<string,*>} Plain object
         */
        UserRequest.toObject = function toObject(message, options) {
            if (!options)
                options = {};
            let object = {};
            if (options.defaults)
                object.userId = "";
            if (message.userId != null && message.hasOwnProperty("userId"))
                object.userId = message.userId;
            return object;
        };

        /**
         * Converts this UserRequest to JSON.
         * @function toJSON
         * @memberof lifelog.UserRequest
         * @instance
         * @returns {Object.<string,*>} JSON object
         */
        UserRequest.prototype.toJSON = function toJSON() {
            return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
        };

        /**
         * Gets the default type url for UserRequest
         * @function getTypeUrl
         * @memberof lifelog.UserRequest
         * @static
         * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns {string} The default type url
         */
        UserRequest.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
            if (typeUrlPrefix === undefined) {
                typeUrlPrefix = "type.googleapis.com";
            }
            return typeUrlPrefix + "/lifelog.UserRequest";
        };

        return UserRequest;
    })();

    lifelog.UserProfile = (function() {

        /**
         * Properties of a UserProfile.
         * @memberof lifelog
         * @interface IUserProfile
         * @property {string|null} [userId] UserProfile userId
         * @property {string|null} [username] UserProfile username
         * @property {string|null} [displayName] UserProfile displayName
         * @property {string|null} [email] UserProfile email
         * @property {string|null} [createdAt] UserProfile createdAt
         * @property {Object.<string,boolean>|null} [settings] UserProfile settings
         */

        /**
         * Constructs a new UserProfile.
         * @memberof lifelog
         * @classdesc Represents a UserProfile.
         * @implements IUserProfile
         * @constructor
         * @param {lifelog.IUserProfile=} [properties] Properties to set
         */
        function UserProfile(properties) {
            this.settings = {};
            if (properties)
                for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                    if (properties[keys[i]] != null)
                        this[keys[i]] = properties[keys[i]];
        }

        /**
         * UserProfile userId.
         * @member {string} userId
         * @memberof lifelog.UserProfile
         * @instance
         */
        UserProfile.prototype.userId = "";

        /**
         * UserProfile username.
         * @member {string} username
         * @memberof lifelog.UserProfile
         * @instance
         */
        UserProfile.prototype.username = "";

        /**
         * UserProfile displayName.
         * @member {string} displayName
         * @memberof lifelog.UserProfile
         * @instance
         */
        UserProfile.prototype.displayName = "";

        /**
         * UserProfile email.
         * @member {string} email
         * @memberof lifelog.UserProfile
         * @instance
         */
        UserProfile.prototype.email = "";

        /**
         * UserProfile createdAt.
         * @member {string} createdAt
         * @memberof lifelog.UserProfile
         * @instance
         */
        UserProfile.prototype.createdAt = "";

        /**
         * UserProfile settings.
         * @member {Object.<string,boolean>} settings
         * @memberof lifelog.UserProfile
         * @instance
         */
        UserProfile.prototype.settings = $util.emptyObject;

        /**
         * Creates a new UserProfile instance using the specified properties.
         * @function create
         * @memberof lifelog.UserProfile
         * @static
         * @param {lifelog.IUserProfile=} [properties] Properties to set
         * @returns {lifelog.UserProfile} UserProfile instance
         */
        UserProfile.create = function create(properties) {
            return new UserProfile(properties);
        };

        /**
         * Encodes the specified UserProfile message. Does not implicitly {@link lifelog.UserProfile.verify|verify} messages.
         * @function encode
         * @memberof lifelog.UserProfile
         * @static
         * @param {lifelog.IUserProfile} message UserProfile message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        UserProfile.encode = function encode(message, writer) {
            if (!writer)
                writer = $Writer.create();
            if (message.userId != null && Object.hasOwnProperty.call(message, "userId"))
                writer.uint32(/* id 1, wireType 2 =*/10).string(message.userId);
            if (message.username != null && Object.hasOwnProperty.call(message, "username"))
                writer.uint32(/* id 2, wireType 2 =*/18).string(message.username);
            if (message.displayName != null && Object.hasOwnProperty.call(message, "displayName"))
                writer.uint32(/* id 3, wireType 2 =*/26).string(message.displayName);
            if (message.email != null && Object.hasOwnProperty.call(message, "email"))
                writer.uint32(/* id 4, wireType 2 =*/34).string(message.email);
            if (message.createdAt != null && Object.hasOwnProperty.call(message, "createdAt"))
                writer.uint32(/* id 5, wireType 2 =*/42).string(message.createdAt);
            if (message.settings != null && Object.hasOwnProperty.call(message, "settings"))
                for (let keys = Object.keys(message.settings), i = 0; i < keys.length; ++i)
                    writer.uint32(/* id 6, wireType 2 =*/50).fork().uint32(/* id 1, wireType 2 =*/10).string(keys[i]).uint32(/* id 2, wireType 0 =*/16).bool(message.settings[keys[i]]).ldelim();
            return writer;
        };

        /**
         * Encodes the specified UserProfile message, length delimited. Does not implicitly {@link lifelog.UserProfile.verify|verify} messages.
         * @function encodeDelimited
         * @memberof lifelog.UserProfile
         * @static
         * @param {lifelog.IUserProfile} message UserProfile message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        UserProfile.encodeDelimited = function encodeDelimited(message, writer) {
            return this.encode(message, writer).ldelim();
        };

        /**
         * Decodes a UserProfile message from the specified reader or buffer.
         * @function decode
         * @memberof lifelog.UserProfile
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @param {number} [length] Message length if known beforehand
         * @returns {lifelog.UserProfile} UserProfile
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        UserProfile.decode = function decode(reader, length, error) {
            if (!(reader instanceof $Reader))
                reader = $Reader.create(reader);
            let end = length === undefined ? reader.len : reader.pos + length, message = new $root.lifelog.UserProfile(), key, value;
            while (reader.pos < end) {
                let tag = reader.uint32();
                if (tag === error)
                    break;
                switch (tag >>> 3) {
                case 1: {
                        message.userId = reader.string();
                        break;
                    }
                case 2: {
                        message.username = reader.string();
                        break;
                    }
                case 3: {
                        message.displayName = reader.string();
                        break;
                    }
                case 4: {
                        message.email = reader.string();
                        break;
                    }
                case 5: {
                        message.createdAt = reader.string();
                        break;
                    }
                case 6: {
                        if (message.settings === $util.emptyObject)
                            message.settings = {};
                        let end2 = reader.uint32() + reader.pos;
                        key = "";
                        value = false;
                        while (reader.pos < end2) {
                            let tag2 = reader.uint32();
                            switch (tag2 >>> 3) {
                            case 1:
                                key = reader.string();
                                break;
                            case 2:
                                value = reader.bool();
                                break;
                            default:
                                reader.skipType(tag2 & 7);
                                break;
                            }
                        }
                        message.settings[key] = value;
                        break;
                    }
                default:
                    reader.skipType(tag & 7);
                    break;
                }
            }
            return message;
        };

        /**
         * Decodes a UserProfile message from the specified reader or buffer, length delimited.
         * @function decodeDelimited
         * @memberof lifelog.UserProfile
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @returns {lifelog.UserProfile} UserProfile
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        UserProfile.decodeDelimited = function decodeDelimited(reader) {
            if (!(reader instanceof $Reader))
                reader = new $Reader(reader);
            return this.decode(reader, reader.uint32());
        };

        /**
         * Verifies a UserProfile message.
         * @function verify
         * @memberof lifelog.UserProfile
         * @static
         * @param {Object.<string,*>} message Plain object to verify
         * @returns {string|null} `null` if valid, otherwise the reason why it is not
         */
        UserProfile.verify = function verify(message) {
            if (typeof message !== "object" || message === null)
                return "object expected";
            if (message.userId != null && message.hasOwnProperty("userId"))
                if (!$util.isString(message.userId))
                    return "userId: string expected";
            if (message.username != null && message.hasOwnProperty("username"))
                if (!$util.isString(message.username))
                    return "username: string expected";
            if (message.displayName != null && message.hasOwnProperty("displayName"))
                if (!$util.isString(message.displayName))
                    return "displayName: string expected";
            if (message.email != null && message.hasOwnProperty("email"))
                if (!$util.isString(message.email))
                    return "email: string expected";
            if (message.createdAt != null && message.hasOwnProperty("createdAt"))
                if (!$util.isString(message.createdAt))
                    return "createdAt: string expected";
            if (message.settings != null && message.hasOwnProperty("settings")) {
                if (!$util.isObject(message.settings))
                    return "settings: object expected";
                let key = Object.keys(message.settings);
                for (let i = 0; i < key.length; ++i)
                    if (typeof message.settings[key[i]] !== "boolean")
                        return "settings: boolean{k:string} expected";
            }
            return null;
        };

        /**
         * Creates a UserProfile message from a plain object. Also converts values to their respective internal types.
         * @function fromObject
         * @memberof lifelog.UserProfile
         * @static
         * @param {Object.<string,*>} object Plain object
         * @returns {lifelog.UserProfile} UserProfile
         */
        UserProfile.fromObject = function fromObject(object) {
            if (object instanceof $root.lifelog.UserProfile)
                return object;
            let message = new $root.lifelog.UserProfile();
            if (object.userId != null)
                message.userId = String(object.userId);
            if (object.username != null)
                message.username = String(object.username);
            if (object.displayName != null)
                message.displayName = String(object.displayName);
            if (object.email != null)
                message.email = String(object.email);
            if (object.createdAt != null)
                message.createdAt = String(object.createdAt);
            if (object.settings) {
                if (typeof object.settings !== "object")
                    throw TypeError(".lifelog.UserProfile.settings: object expected");
                message.settings = {};
                for (let keys = Object.keys(object.settings), i = 0; i < keys.length; ++i)
                    message.settings[keys[i]] = Boolean(object.settings[keys[i]]);
            }
            return message;
        };

        /**
         * Creates a plain object from a UserProfile message. Also converts values to other types if specified.
         * @function toObject
         * @memberof lifelog.UserProfile
         * @static
         * @param {lifelog.UserProfile} message UserProfile
         * @param {$protobuf.IConversionOptions} [options] Conversion options
         * @returns {Object.<string,*>} Plain object
         */
        UserProfile.toObject = function toObject(message, options) {
            if (!options)
                options = {};
            let object = {};
            if (options.objects || options.defaults)
                object.settings = {};
            if (options.defaults) {
                object.userId = "";
                object.username = "";
                object.displayName = "";
                object.email = "";
                object.createdAt = "";
            }
            if (message.userId != null && message.hasOwnProperty("userId"))
                object.userId = message.userId;
            if (message.username != null && message.hasOwnProperty("username"))
                object.username = message.username;
            if (message.displayName != null && message.hasOwnProperty("displayName"))
                object.displayName = message.displayName;
            if (message.email != null && message.hasOwnProperty("email"))
                object.email = message.email;
            if (message.createdAt != null && message.hasOwnProperty("createdAt"))
                object.createdAt = message.createdAt;
            let keys2;
            if (message.settings && (keys2 = Object.keys(message.settings)).length) {
                object.settings = {};
                for (let j = 0; j < keys2.length; ++j)
                    object.settings[keys2[j]] = message.settings[keys2[j]];
            }
            return object;
        };

        /**
         * Converts this UserProfile to JSON.
         * @function toJSON
         * @memberof lifelog.UserProfile
         * @instance
         * @returns {Object.<string,*>} JSON object
         */
        UserProfile.prototype.toJSON = function toJSON() {
            return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
        };

        /**
         * Gets the default type url for UserProfile
         * @function getTypeUrl
         * @memberof lifelog.UserProfile
         * @static
         * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns {string} The default type url
         */
        UserProfile.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
            if (typeUrlPrefix === undefined) {
                typeUrlPrefix = "type.googleapis.com";
            }
            return typeUrlPrefix + "/lifelog.UserProfile";
        };

        return UserProfile;
    })();

    lifelog.LoggerStatusRequest = (function() {

        /**
         * Properties of a LoggerStatusRequest.
         * @memberof lifelog
         * @interface ILoggerStatusRequest
         * @property {Array.<string>|null} [loggerNames] LoggerStatusRequest loggerNames
         */

        /**
         * Constructs a new LoggerStatusRequest.
         * @memberof lifelog
         * @classdesc Represents a LoggerStatusRequest.
         * @implements ILoggerStatusRequest
         * @constructor
         * @param {lifelog.ILoggerStatusRequest=} [properties] Properties to set
         */
        function LoggerStatusRequest(properties) {
            this.loggerNames = [];
            if (properties)
                for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                    if (properties[keys[i]] != null)
                        this[keys[i]] = properties[keys[i]];
        }

        /**
         * LoggerStatusRequest loggerNames.
         * @member {Array.<string>} loggerNames
         * @memberof lifelog.LoggerStatusRequest
         * @instance
         */
        LoggerStatusRequest.prototype.loggerNames = $util.emptyArray;

        /**
         * Creates a new LoggerStatusRequest instance using the specified properties.
         * @function create
         * @memberof lifelog.LoggerStatusRequest
         * @static
         * @param {lifelog.ILoggerStatusRequest=} [properties] Properties to set
         * @returns {lifelog.LoggerStatusRequest} LoggerStatusRequest instance
         */
        LoggerStatusRequest.create = function create(properties) {
            return new LoggerStatusRequest(properties);
        };

        /**
         * Encodes the specified LoggerStatusRequest message. Does not implicitly {@link lifelog.LoggerStatusRequest.verify|verify} messages.
         * @function encode
         * @memberof lifelog.LoggerStatusRequest
         * @static
         * @param {lifelog.ILoggerStatusRequest} message LoggerStatusRequest message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        LoggerStatusRequest.encode = function encode(message, writer) {
            if (!writer)
                writer = $Writer.create();
            if (message.loggerNames != null && message.loggerNames.length)
                for (let i = 0; i < message.loggerNames.length; ++i)
                    writer.uint32(/* id 1, wireType 2 =*/10).string(message.loggerNames[i]);
            return writer;
        };

        /**
         * Encodes the specified LoggerStatusRequest message, length delimited. Does not implicitly {@link lifelog.LoggerStatusRequest.verify|verify} messages.
         * @function encodeDelimited
         * @memberof lifelog.LoggerStatusRequest
         * @static
         * @param {lifelog.ILoggerStatusRequest} message LoggerStatusRequest message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        LoggerStatusRequest.encodeDelimited = function encodeDelimited(message, writer) {
            return this.encode(message, writer).ldelim();
        };

        /**
         * Decodes a LoggerStatusRequest message from the specified reader or buffer.
         * @function decode
         * @memberof lifelog.LoggerStatusRequest
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @param {number} [length] Message length if known beforehand
         * @returns {lifelog.LoggerStatusRequest} LoggerStatusRequest
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        LoggerStatusRequest.decode = function decode(reader, length, error) {
            if (!(reader instanceof $Reader))
                reader = $Reader.create(reader);
            let end = length === undefined ? reader.len : reader.pos + length, message = new $root.lifelog.LoggerStatusRequest();
            while (reader.pos < end) {
                let tag = reader.uint32();
                if (tag === error)
                    break;
                switch (tag >>> 3) {
                case 1: {
                        if (!(message.loggerNames && message.loggerNames.length))
                            message.loggerNames = [];
                        message.loggerNames.push(reader.string());
                        break;
                    }
                default:
                    reader.skipType(tag & 7);
                    break;
                }
            }
            return message;
        };

        /**
         * Decodes a LoggerStatusRequest message from the specified reader or buffer, length delimited.
         * @function decodeDelimited
         * @memberof lifelog.LoggerStatusRequest
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @returns {lifelog.LoggerStatusRequest} LoggerStatusRequest
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        LoggerStatusRequest.decodeDelimited = function decodeDelimited(reader) {
            if (!(reader instanceof $Reader))
                reader = new $Reader(reader);
            return this.decode(reader, reader.uint32());
        };

        /**
         * Verifies a LoggerStatusRequest message.
         * @function verify
         * @memberof lifelog.LoggerStatusRequest
         * @static
         * @param {Object.<string,*>} message Plain object to verify
         * @returns {string|null} `null` if valid, otherwise the reason why it is not
         */
        LoggerStatusRequest.verify = function verify(message) {
            if (typeof message !== "object" || message === null)
                return "object expected";
            if (message.loggerNames != null && message.hasOwnProperty("loggerNames")) {
                if (!Array.isArray(message.loggerNames))
                    return "loggerNames: array expected";
                for (let i = 0; i < message.loggerNames.length; ++i)
                    if (!$util.isString(message.loggerNames[i]))
                        return "loggerNames: string[] expected";
            }
            return null;
        };

        /**
         * Creates a LoggerStatusRequest message from a plain object. Also converts values to their respective internal types.
         * @function fromObject
         * @memberof lifelog.LoggerStatusRequest
         * @static
         * @param {Object.<string,*>} object Plain object
         * @returns {lifelog.LoggerStatusRequest} LoggerStatusRequest
         */
        LoggerStatusRequest.fromObject = function fromObject(object) {
            if (object instanceof $root.lifelog.LoggerStatusRequest)
                return object;
            let message = new $root.lifelog.LoggerStatusRequest();
            if (object.loggerNames) {
                if (!Array.isArray(object.loggerNames))
                    throw TypeError(".lifelog.LoggerStatusRequest.loggerNames: array expected");
                message.loggerNames = [];
                for (let i = 0; i < object.loggerNames.length; ++i)
                    message.loggerNames[i] = String(object.loggerNames[i]);
            }
            return message;
        };

        /**
         * Creates a plain object from a LoggerStatusRequest message. Also converts values to other types if specified.
         * @function toObject
         * @memberof lifelog.LoggerStatusRequest
         * @static
         * @param {lifelog.LoggerStatusRequest} message LoggerStatusRequest
         * @param {$protobuf.IConversionOptions} [options] Conversion options
         * @returns {Object.<string,*>} Plain object
         */
        LoggerStatusRequest.toObject = function toObject(message, options) {
            if (!options)
                options = {};
            let object = {};
            if (options.arrays || options.defaults)
                object.loggerNames = [];
            if (message.loggerNames && message.loggerNames.length) {
                object.loggerNames = [];
                for (let j = 0; j < message.loggerNames.length; ++j)
                    object.loggerNames[j] = message.loggerNames[j];
            }
            return object;
        };

        /**
         * Converts this LoggerStatusRequest to JSON.
         * @function toJSON
         * @memberof lifelog.LoggerStatusRequest
         * @instance
         * @returns {Object.<string,*>} JSON object
         */
        LoggerStatusRequest.prototype.toJSON = function toJSON() {
            return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
        };

        /**
         * Gets the default type url for LoggerStatusRequest
         * @function getTypeUrl
         * @memberof lifelog.LoggerStatusRequest
         * @static
         * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns {string} The default type url
         */
        LoggerStatusRequest.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
            if (typeUrlPrefix === undefined) {
                typeUrlPrefix = "type.googleapis.com";
            }
            return typeUrlPrefix + "/lifelog.LoggerStatusRequest";
        };

        return LoggerStatusRequest;
    })();

    lifelog.LoggerStatus = (function() {

        /**
         * Properties of a LoggerStatus.
         * @memberof lifelog
         * @interface ILoggerStatus
         * @property {string|null} [name] LoggerStatus name
         * @property {boolean|null} [enabled] LoggerStatus enabled
         * @property {boolean|null} [running] LoggerStatus running
         * @property {string|null} [lastActive] LoggerStatus lastActive
         * @property {number|Long|null} [dataPoints] LoggerStatus dataPoints
         * @property {string|null} [error] LoggerStatus error
         */

        /**
         * Constructs a new LoggerStatus.
         * @memberof lifelog
         * @classdesc Represents a LoggerStatus.
         * @implements ILoggerStatus
         * @constructor
         * @param {lifelog.ILoggerStatus=} [properties] Properties to set
         */
        function LoggerStatus(properties) {
            if (properties)
                for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                    if (properties[keys[i]] != null)
                        this[keys[i]] = properties[keys[i]];
        }

        /**
         * LoggerStatus name.
         * @member {string} name
         * @memberof lifelog.LoggerStatus
         * @instance
         */
        LoggerStatus.prototype.name = "";

        /**
         * LoggerStatus enabled.
         * @member {boolean} enabled
         * @memberof lifelog.LoggerStatus
         * @instance
         */
        LoggerStatus.prototype.enabled = false;

        /**
         * LoggerStatus running.
         * @member {boolean} running
         * @memberof lifelog.LoggerStatus
         * @instance
         */
        LoggerStatus.prototype.running = false;

        /**
         * LoggerStatus lastActive.
         * @member {string} lastActive
         * @memberof lifelog.LoggerStatus
         * @instance
         */
        LoggerStatus.prototype.lastActive = "";

        /**
         * LoggerStatus dataPoints.
         * @member {number|Long} dataPoints
         * @memberof lifelog.LoggerStatus
         * @instance
         */
        LoggerStatus.prototype.dataPoints = $util.Long ? $util.Long.fromBits(0,0,false) : 0;

        /**
         * LoggerStatus error.
         * @member {string} error
         * @memberof lifelog.LoggerStatus
         * @instance
         */
        LoggerStatus.prototype.error = "";

        /**
         * Creates a new LoggerStatus instance using the specified properties.
         * @function create
         * @memberof lifelog.LoggerStatus
         * @static
         * @param {lifelog.ILoggerStatus=} [properties] Properties to set
         * @returns {lifelog.LoggerStatus} LoggerStatus instance
         */
        LoggerStatus.create = function create(properties) {
            return new LoggerStatus(properties);
        };

        /**
         * Encodes the specified LoggerStatus message. Does not implicitly {@link lifelog.LoggerStatus.verify|verify} messages.
         * @function encode
         * @memberof lifelog.LoggerStatus
         * @static
         * @param {lifelog.ILoggerStatus} message LoggerStatus message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        LoggerStatus.encode = function encode(message, writer) {
            if (!writer)
                writer = $Writer.create();
            if (message.name != null && Object.hasOwnProperty.call(message, "name"))
                writer.uint32(/* id 1, wireType 2 =*/10).string(message.name);
            if (message.enabled != null && Object.hasOwnProperty.call(message, "enabled"))
                writer.uint32(/* id 2, wireType 0 =*/16).bool(message.enabled);
            if (message.running != null && Object.hasOwnProperty.call(message, "running"))
                writer.uint32(/* id 3, wireType 0 =*/24).bool(message.running);
            if (message.lastActive != null && Object.hasOwnProperty.call(message, "lastActive"))
                writer.uint32(/* id 4, wireType 2 =*/34).string(message.lastActive);
            if (message.dataPoints != null && Object.hasOwnProperty.call(message, "dataPoints"))
                writer.uint32(/* id 5, wireType 0 =*/40).int64(message.dataPoints);
            if (message.error != null && Object.hasOwnProperty.call(message, "error"))
                writer.uint32(/* id 6, wireType 2 =*/50).string(message.error);
            return writer;
        };

        /**
         * Encodes the specified LoggerStatus message, length delimited. Does not implicitly {@link lifelog.LoggerStatus.verify|verify} messages.
         * @function encodeDelimited
         * @memberof lifelog.LoggerStatus
         * @static
         * @param {lifelog.ILoggerStatus} message LoggerStatus message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        LoggerStatus.encodeDelimited = function encodeDelimited(message, writer) {
            return this.encode(message, writer).ldelim();
        };

        /**
         * Decodes a LoggerStatus message from the specified reader or buffer.
         * @function decode
         * @memberof lifelog.LoggerStatus
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @param {number} [length] Message length if known beforehand
         * @returns {lifelog.LoggerStatus} LoggerStatus
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        LoggerStatus.decode = function decode(reader, length, error) {
            if (!(reader instanceof $Reader))
                reader = $Reader.create(reader);
            let end = length === undefined ? reader.len : reader.pos + length, message = new $root.lifelog.LoggerStatus();
            while (reader.pos < end) {
                let tag = reader.uint32();
                if (tag === error)
                    break;
                switch (tag >>> 3) {
                case 1: {
                        message.name = reader.string();
                        break;
                    }
                case 2: {
                        message.enabled = reader.bool();
                        break;
                    }
                case 3: {
                        message.running = reader.bool();
                        break;
                    }
                case 4: {
                        message.lastActive = reader.string();
                        break;
                    }
                case 5: {
                        message.dataPoints = reader.int64();
                        break;
                    }
                case 6: {
                        message.error = reader.string();
                        break;
                    }
                default:
                    reader.skipType(tag & 7);
                    break;
                }
            }
            return message;
        };

        /**
         * Decodes a LoggerStatus message from the specified reader or buffer, length delimited.
         * @function decodeDelimited
         * @memberof lifelog.LoggerStatus
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @returns {lifelog.LoggerStatus} LoggerStatus
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        LoggerStatus.decodeDelimited = function decodeDelimited(reader) {
            if (!(reader instanceof $Reader))
                reader = new $Reader(reader);
            return this.decode(reader, reader.uint32());
        };

        /**
         * Verifies a LoggerStatus message.
         * @function verify
         * @memberof lifelog.LoggerStatus
         * @static
         * @param {Object.<string,*>} message Plain object to verify
         * @returns {string|null} `null` if valid, otherwise the reason why it is not
         */
        LoggerStatus.verify = function verify(message) {
            if (typeof message !== "object" || message === null)
                return "object expected";
            if (message.name != null && message.hasOwnProperty("name"))
                if (!$util.isString(message.name))
                    return "name: string expected";
            if (message.enabled != null && message.hasOwnProperty("enabled"))
                if (typeof message.enabled !== "boolean")
                    return "enabled: boolean expected";
            if (message.running != null && message.hasOwnProperty("running"))
                if (typeof message.running !== "boolean")
                    return "running: boolean expected";
            if (message.lastActive != null && message.hasOwnProperty("lastActive"))
                if (!$util.isString(message.lastActive))
                    return "lastActive: string expected";
            if (message.dataPoints != null && message.hasOwnProperty("dataPoints"))
                if (!$util.isInteger(message.dataPoints) && !(message.dataPoints && $util.isInteger(message.dataPoints.low) && $util.isInteger(message.dataPoints.high)))
                    return "dataPoints: integer|Long expected";
            if (message.error != null && message.hasOwnProperty("error"))
                if (!$util.isString(message.error))
                    return "error: string expected";
            return null;
        };

        /**
         * Creates a LoggerStatus message from a plain object. Also converts values to their respective internal types.
         * @function fromObject
         * @memberof lifelog.LoggerStatus
         * @static
         * @param {Object.<string,*>} object Plain object
         * @returns {lifelog.LoggerStatus} LoggerStatus
         */
        LoggerStatus.fromObject = function fromObject(object) {
            if (object instanceof $root.lifelog.LoggerStatus)
                return object;
            let message = new $root.lifelog.LoggerStatus();
            if (object.name != null)
                message.name = String(object.name);
            if (object.enabled != null)
                message.enabled = Boolean(object.enabled);
            if (object.running != null)
                message.running = Boolean(object.running);
            if (object.lastActive != null)
                message.lastActive = String(object.lastActive);
            if (object.dataPoints != null)
                if ($util.Long)
                    (message.dataPoints = $util.Long.fromValue(object.dataPoints)).unsigned = false;
                else if (typeof object.dataPoints === "string")
                    message.dataPoints = parseInt(object.dataPoints, 10);
                else if (typeof object.dataPoints === "number")
                    message.dataPoints = object.dataPoints;
                else if (typeof object.dataPoints === "object")
                    message.dataPoints = new $util.LongBits(object.dataPoints.low >>> 0, object.dataPoints.high >>> 0).toNumber();
            if (object.error != null)
                message.error = String(object.error);
            return message;
        };

        /**
         * Creates a plain object from a LoggerStatus message. Also converts values to other types if specified.
         * @function toObject
         * @memberof lifelog.LoggerStatus
         * @static
         * @param {lifelog.LoggerStatus} message LoggerStatus
         * @param {$protobuf.IConversionOptions} [options] Conversion options
         * @returns {Object.<string,*>} Plain object
         */
        LoggerStatus.toObject = function toObject(message, options) {
            if (!options)
                options = {};
            let object = {};
            if (options.defaults) {
                object.name = "";
                object.enabled = false;
                object.running = false;
                object.lastActive = "";
                if ($util.Long) {
                    let long = new $util.Long(0, 0, false);
                    object.dataPoints = options.longs === String ? long.toString() : options.longs === Number ? long.toNumber() : long;
                } else
                    object.dataPoints = options.longs === String ? "0" : 0;
                object.error = "";
            }
            if (message.name != null && message.hasOwnProperty("name"))
                object.name = message.name;
            if (message.enabled != null && message.hasOwnProperty("enabled"))
                object.enabled = message.enabled;
            if (message.running != null && message.hasOwnProperty("running"))
                object.running = message.running;
            if (message.lastActive != null && message.hasOwnProperty("lastActive"))
                object.lastActive = message.lastActive;
            if (message.dataPoints != null && message.hasOwnProperty("dataPoints"))
                if (typeof message.dataPoints === "number")
                    object.dataPoints = options.longs === String ? String(message.dataPoints) : message.dataPoints;
                else
                    object.dataPoints = options.longs === String ? $util.Long.prototype.toString.call(message.dataPoints) : options.longs === Number ? new $util.LongBits(message.dataPoints.low >>> 0, message.dataPoints.high >>> 0).toNumber() : message.dataPoints;
            if (message.error != null && message.hasOwnProperty("error"))
                object.error = message.error;
            return object;
        };

        /**
         * Converts this LoggerStatus to JSON.
         * @function toJSON
         * @memberof lifelog.LoggerStatus
         * @instance
         * @returns {Object.<string,*>} JSON object
         */
        LoggerStatus.prototype.toJSON = function toJSON() {
            return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
        };

        /**
         * Gets the default type url for LoggerStatus
         * @function getTypeUrl
         * @memberof lifelog.LoggerStatus
         * @static
         * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns {string} The default type url
         */
        LoggerStatus.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
            if (typeUrlPrefix === undefined) {
                typeUrlPrefix = "type.googleapis.com";
            }
            return typeUrlPrefix + "/lifelog.LoggerStatus";
        };

        return LoggerStatus;
    })();

    lifelog.LoggerStatusResponse = (function() {

        /**
         * Properties of a LoggerStatusResponse.
         * @memberof lifelog
         * @interface ILoggerStatusResponse
         * @property {Array.<lifelog.ILoggerStatus>|null} [loggers] LoggerStatusResponse loggers
         * @property {Object.<string,string>|null} [systemStats] LoggerStatusResponse systemStats
         */

        /**
         * Constructs a new LoggerStatusResponse.
         * @memberof lifelog
         * @classdesc Represents a LoggerStatusResponse.
         * @implements ILoggerStatusResponse
         * @constructor
         * @param {lifelog.ILoggerStatusResponse=} [properties] Properties to set
         */
        function LoggerStatusResponse(properties) {
            this.loggers = [];
            this.systemStats = {};
            if (properties)
                for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                    if (properties[keys[i]] != null)
                        this[keys[i]] = properties[keys[i]];
        }

        /**
         * LoggerStatusResponse loggers.
         * @member {Array.<lifelog.ILoggerStatus>} loggers
         * @memberof lifelog.LoggerStatusResponse
         * @instance
         */
        LoggerStatusResponse.prototype.loggers = $util.emptyArray;

        /**
         * LoggerStatusResponse systemStats.
         * @member {Object.<string,string>} systemStats
         * @memberof lifelog.LoggerStatusResponse
         * @instance
         */
        LoggerStatusResponse.prototype.systemStats = $util.emptyObject;

        /**
         * Creates a new LoggerStatusResponse instance using the specified properties.
         * @function create
         * @memberof lifelog.LoggerStatusResponse
         * @static
         * @param {lifelog.ILoggerStatusResponse=} [properties] Properties to set
         * @returns {lifelog.LoggerStatusResponse} LoggerStatusResponse instance
         */
        LoggerStatusResponse.create = function create(properties) {
            return new LoggerStatusResponse(properties);
        };

        /**
         * Encodes the specified LoggerStatusResponse message. Does not implicitly {@link lifelog.LoggerStatusResponse.verify|verify} messages.
         * @function encode
         * @memberof lifelog.LoggerStatusResponse
         * @static
         * @param {lifelog.ILoggerStatusResponse} message LoggerStatusResponse message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        LoggerStatusResponse.encode = function encode(message, writer) {
            if (!writer)
                writer = $Writer.create();
            if (message.loggers != null && message.loggers.length)
                for (let i = 0; i < message.loggers.length; ++i)
                    $root.lifelog.LoggerStatus.encode(message.loggers[i], writer.uint32(/* id 1, wireType 2 =*/10).fork()).ldelim();
            if (message.systemStats != null && Object.hasOwnProperty.call(message, "systemStats"))
                for (let keys = Object.keys(message.systemStats), i = 0; i < keys.length; ++i)
                    writer.uint32(/* id 2, wireType 2 =*/18).fork().uint32(/* id 1, wireType 2 =*/10).string(keys[i]).uint32(/* id 2, wireType 2 =*/18).string(message.systemStats[keys[i]]).ldelim();
            return writer;
        };

        /**
         * Encodes the specified LoggerStatusResponse message, length delimited. Does not implicitly {@link lifelog.LoggerStatusResponse.verify|verify} messages.
         * @function encodeDelimited
         * @memberof lifelog.LoggerStatusResponse
         * @static
         * @param {lifelog.ILoggerStatusResponse} message LoggerStatusResponse message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        LoggerStatusResponse.encodeDelimited = function encodeDelimited(message, writer) {
            return this.encode(message, writer).ldelim();
        };

        /**
         * Decodes a LoggerStatusResponse message from the specified reader or buffer.
         * @function decode
         * @memberof lifelog.LoggerStatusResponse
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @param {number} [length] Message length if known beforehand
         * @returns {lifelog.LoggerStatusResponse} LoggerStatusResponse
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        LoggerStatusResponse.decode = function decode(reader, length, error) {
            if (!(reader instanceof $Reader))
                reader = $Reader.create(reader);
            let end = length === undefined ? reader.len : reader.pos + length, message = new $root.lifelog.LoggerStatusResponse(), key, value;
            while (reader.pos < end) {
                let tag = reader.uint32();
                if (tag === error)
                    break;
                switch (tag >>> 3) {
                case 1: {
                        if (!(message.loggers && message.loggers.length))
                            message.loggers = [];
                        message.loggers.push($root.lifelog.LoggerStatus.decode(reader, reader.uint32()));
                        break;
                    }
                case 2: {
                        if (message.systemStats === $util.emptyObject)
                            message.systemStats = {};
                        let end2 = reader.uint32() + reader.pos;
                        key = "";
                        value = "";
                        while (reader.pos < end2) {
                            let tag2 = reader.uint32();
                            switch (tag2 >>> 3) {
                            case 1:
                                key = reader.string();
                                break;
                            case 2:
                                value = reader.string();
                                break;
                            default:
                                reader.skipType(tag2 & 7);
                                break;
                            }
                        }
                        message.systemStats[key] = value;
                        break;
                    }
                default:
                    reader.skipType(tag & 7);
                    break;
                }
            }
            return message;
        };

        /**
         * Decodes a LoggerStatusResponse message from the specified reader or buffer, length delimited.
         * @function decodeDelimited
         * @memberof lifelog.LoggerStatusResponse
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @returns {lifelog.LoggerStatusResponse} LoggerStatusResponse
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        LoggerStatusResponse.decodeDelimited = function decodeDelimited(reader) {
            if (!(reader instanceof $Reader))
                reader = new $Reader(reader);
            return this.decode(reader, reader.uint32());
        };

        /**
         * Verifies a LoggerStatusResponse message.
         * @function verify
         * @memberof lifelog.LoggerStatusResponse
         * @static
         * @param {Object.<string,*>} message Plain object to verify
         * @returns {string|null} `null` if valid, otherwise the reason why it is not
         */
        LoggerStatusResponse.verify = function verify(message) {
            if (typeof message !== "object" || message === null)
                return "object expected";
            if (message.loggers != null && message.hasOwnProperty("loggers")) {
                if (!Array.isArray(message.loggers))
                    return "loggers: array expected";
                for (let i = 0; i < message.loggers.length; ++i) {
                    let error = $root.lifelog.LoggerStatus.verify(message.loggers[i]);
                    if (error)
                        return "loggers." + error;
                }
            }
            if (message.systemStats != null && message.hasOwnProperty("systemStats")) {
                if (!$util.isObject(message.systemStats))
                    return "systemStats: object expected";
                let key = Object.keys(message.systemStats);
                for (let i = 0; i < key.length; ++i)
                    if (!$util.isString(message.systemStats[key[i]]))
                        return "systemStats: string{k:string} expected";
            }
            return null;
        };

        /**
         * Creates a LoggerStatusResponse message from a plain object. Also converts values to their respective internal types.
         * @function fromObject
         * @memberof lifelog.LoggerStatusResponse
         * @static
         * @param {Object.<string,*>} object Plain object
         * @returns {lifelog.LoggerStatusResponse} LoggerStatusResponse
         */
        LoggerStatusResponse.fromObject = function fromObject(object) {
            if (object instanceof $root.lifelog.LoggerStatusResponse)
                return object;
            let message = new $root.lifelog.LoggerStatusResponse();
            if (object.loggers) {
                if (!Array.isArray(object.loggers))
                    throw TypeError(".lifelog.LoggerStatusResponse.loggers: array expected");
                message.loggers = [];
                for (let i = 0; i < object.loggers.length; ++i) {
                    if (typeof object.loggers[i] !== "object")
                        throw TypeError(".lifelog.LoggerStatusResponse.loggers: object expected");
                    message.loggers[i] = $root.lifelog.LoggerStatus.fromObject(object.loggers[i]);
                }
            }
            if (object.systemStats) {
                if (typeof object.systemStats !== "object")
                    throw TypeError(".lifelog.LoggerStatusResponse.systemStats: object expected");
                message.systemStats = {};
                for (let keys = Object.keys(object.systemStats), i = 0; i < keys.length; ++i)
                    message.systemStats[keys[i]] = String(object.systemStats[keys[i]]);
            }
            return message;
        };

        /**
         * Creates a plain object from a LoggerStatusResponse message. Also converts values to other types if specified.
         * @function toObject
         * @memberof lifelog.LoggerStatusResponse
         * @static
         * @param {lifelog.LoggerStatusResponse} message LoggerStatusResponse
         * @param {$protobuf.IConversionOptions} [options] Conversion options
         * @returns {Object.<string,*>} Plain object
         */
        LoggerStatusResponse.toObject = function toObject(message, options) {
            if (!options)
                options = {};
            let object = {};
            if (options.arrays || options.defaults)
                object.loggers = [];
            if (options.objects || options.defaults)
                object.systemStats = {};
            if (message.loggers && message.loggers.length) {
                object.loggers = [];
                for (let j = 0; j < message.loggers.length; ++j)
                    object.loggers[j] = $root.lifelog.LoggerStatus.toObject(message.loggers[j], options);
            }
            let keys2;
            if (message.systemStats && (keys2 = Object.keys(message.systemStats)).length) {
                object.systemStats = {};
                for (let j = 0; j < keys2.length; ++j)
                    object.systemStats[keys2[j]] = message.systemStats[keys2[j]];
            }
            return object;
        };

        /**
         * Converts this LoggerStatusResponse to JSON.
         * @function toJSON
         * @memberof lifelog.LoggerStatusResponse
         * @instance
         * @returns {Object.<string,*>} JSON object
         */
        LoggerStatusResponse.prototype.toJSON = function toJSON() {
            return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
        };

        /**
         * Gets the default type url for LoggerStatusResponse
         * @function getTypeUrl
         * @memberof lifelog.LoggerStatusResponse
         * @static
         * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns {string} The default type url
         */
        LoggerStatusResponse.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
            if (typeUrlPrefix === undefined) {
                typeUrlPrefix = "type.googleapis.com";
            }
            return typeUrlPrefix + "/lifelog.LoggerStatusResponse";
        };

        return LoggerStatusResponse;
    })();

    lifelog.ToggleLoggerRequest = (function() {

        /**
         * Properties of a ToggleLoggerRequest.
         * @memberof lifelog
         * @interface IToggleLoggerRequest
         * @property {string|null} [loggerName] ToggleLoggerRequest loggerName
         * @property {boolean|null} [enable] ToggleLoggerRequest enable
         */

        /**
         * Constructs a new ToggleLoggerRequest.
         * @memberof lifelog
         * @classdesc Represents a ToggleLoggerRequest.
         * @implements IToggleLoggerRequest
         * @constructor
         * @param {lifelog.IToggleLoggerRequest=} [properties] Properties to set
         */
        function ToggleLoggerRequest(properties) {
            if (properties)
                for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                    if (properties[keys[i]] != null)
                        this[keys[i]] = properties[keys[i]];
        }

        /**
         * ToggleLoggerRequest loggerName.
         * @member {string} loggerName
         * @memberof lifelog.ToggleLoggerRequest
         * @instance
         */
        ToggleLoggerRequest.prototype.loggerName = "";

        /**
         * ToggleLoggerRequest enable.
         * @member {boolean} enable
         * @memberof lifelog.ToggleLoggerRequest
         * @instance
         */
        ToggleLoggerRequest.prototype.enable = false;

        /**
         * Creates a new ToggleLoggerRequest instance using the specified properties.
         * @function create
         * @memberof lifelog.ToggleLoggerRequest
         * @static
         * @param {lifelog.IToggleLoggerRequest=} [properties] Properties to set
         * @returns {lifelog.ToggleLoggerRequest} ToggleLoggerRequest instance
         */
        ToggleLoggerRequest.create = function create(properties) {
            return new ToggleLoggerRequest(properties);
        };

        /**
         * Encodes the specified ToggleLoggerRequest message. Does not implicitly {@link lifelog.ToggleLoggerRequest.verify|verify} messages.
         * @function encode
         * @memberof lifelog.ToggleLoggerRequest
         * @static
         * @param {lifelog.IToggleLoggerRequest} message ToggleLoggerRequest message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        ToggleLoggerRequest.encode = function encode(message, writer) {
            if (!writer)
                writer = $Writer.create();
            if (message.loggerName != null && Object.hasOwnProperty.call(message, "loggerName"))
                writer.uint32(/* id 1, wireType 2 =*/10).string(message.loggerName);
            if (message.enable != null && Object.hasOwnProperty.call(message, "enable"))
                writer.uint32(/* id 2, wireType 0 =*/16).bool(message.enable);
            return writer;
        };

        /**
         * Encodes the specified ToggleLoggerRequest message, length delimited. Does not implicitly {@link lifelog.ToggleLoggerRequest.verify|verify} messages.
         * @function encodeDelimited
         * @memberof lifelog.ToggleLoggerRequest
         * @static
         * @param {lifelog.IToggleLoggerRequest} message ToggleLoggerRequest message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        ToggleLoggerRequest.encodeDelimited = function encodeDelimited(message, writer) {
            return this.encode(message, writer).ldelim();
        };

        /**
         * Decodes a ToggleLoggerRequest message from the specified reader or buffer.
         * @function decode
         * @memberof lifelog.ToggleLoggerRequest
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @param {number} [length] Message length if known beforehand
         * @returns {lifelog.ToggleLoggerRequest} ToggleLoggerRequest
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        ToggleLoggerRequest.decode = function decode(reader, length, error) {
            if (!(reader instanceof $Reader))
                reader = $Reader.create(reader);
            let end = length === undefined ? reader.len : reader.pos + length, message = new $root.lifelog.ToggleLoggerRequest();
            while (reader.pos < end) {
                let tag = reader.uint32();
                if (tag === error)
                    break;
                switch (tag >>> 3) {
                case 1: {
                        message.loggerName = reader.string();
                        break;
                    }
                case 2: {
                        message.enable = reader.bool();
                        break;
                    }
                default:
                    reader.skipType(tag & 7);
                    break;
                }
            }
            return message;
        };

        /**
         * Decodes a ToggleLoggerRequest message from the specified reader or buffer, length delimited.
         * @function decodeDelimited
         * @memberof lifelog.ToggleLoggerRequest
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @returns {lifelog.ToggleLoggerRequest} ToggleLoggerRequest
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        ToggleLoggerRequest.decodeDelimited = function decodeDelimited(reader) {
            if (!(reader instanceof $Reader))
                reader = new $Reader(reader);
            return this.decode(reader, reader.uint32());
        };

        /**
         * Verifies a ToggleLoggerRequest message.
         * @function verify
         * @memberof lifelog.ToggleLoggerRequest
         * @static
         * @param {Object.<string,*>} message Plain object to verify
         * @returns {string|null} `null` if valid, otherwise the reason why it is not
         */
        ToggleLoggerRequest.verify = function verify(message) {
            if (typeof message !== "object" || message === null)
                return "object expected";
            if (message.loggerName != null && message.hasOwnProperty("loggerName"))
                if (!$util.isString(message.loggerName))
                    return "loggerName: string expected";
            if (message.enable != null && message.hasOwnProperty("enable"))
                if (typeof message.enable !== "boolean")
                    return "enable: boolean expected";
            return null;
        };

        /**
         * Creates a ToggleLoggerRequest message from a plain object. Also converts values to their respective internal types.
         * @function fromObject
         * @memberof lifelog.ToggleLoggerRequest
         * @static
         * @param {Object.<string,*>} object Plain object
         * @returns {lifelog.ToggleLoggerRequest} ToggleLoggerRequest
         */
        ToggleLoggerRequest.fromObject = function fromObject(object) {
            if (object instanceof $root.lifelog.ToggleLoggerRequest)
                return object;
            let message = new $root.lifelog.ToggleLoggerRequest();
            if (object.loggerName != null)
                message.loggerName = String(object.loggerName);
            if (object.enable != null)
                message.enable = Boolean(object.enable);
            return message;
        };

        /**
         * Creates a plain object from a ToggleLoggerRequest message. Also converts values to other types if specified.
         * @function toObject
         * @memberof lifelog.ToggleLoggerRequest
         * @static
         * @param {lifelog.ToggleLoggerRequest} message ToggleLoggerRequest
         * @param {$protobuf.IConversionOptions} [options] Conversion options
         * @returns {Object.<string,*>} Plain object
         */
        ToggleLoggerRequest.toObject = function toObject(message, options) {
            if (!options)
                options = {};
            let object = {};
            if (options.defaults) {
                object.loggerName = "";
                object.enable = false;
            }
            if (message.loggerName != null && message.hasOwnProperty("loggerName"))
                object.loggerName = message.loggerName;
            if (message.enable != null && message.hasOwnProperty("enable"))
                object.enable = message.enable;
            return object;
        };

        /**
         * Converts this ToggleLoggerRequest to JSON.
         * @function toJSON
         * @memberof lifelog.ToggleLoggerRequest
         * @instance
         * @returns {Object.<string,*>} JSON object
         */
        ToggleLoggerRequest.prototype.toJSON = function toJSON() {
            return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
        };

        /**
         * Gets the default type url for ToggleLoggerRequest
         * @function getTypeUrl
         * @memberof lifelog.ToggleLoggerRequest
         * @static
         * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns {string} The default type url
         */
        ToggleLoggerRequest.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
            if (typeUrlPrefix === undefined) {
                typeUrlPrefix = "type.googleapis.com";
            }
            return typeUrlPrefix + "/lifelog.ToggleLoggerRequest";
        };

        return ToggleLoggerRequest;
    })();

    lifelog.ToggleLoggerResponse = (function() {

        /**
         * Properties of a ToggleLoggerResponse.
         * @memberof lifelog
         * @interface IToggleLoggerResponse
         * @property {boolean|null} [success] ToggleLoggerResponse success
         * @property {string|null} [errorMessage] ToggleLoggerResponse errorMessage
         * @property {lifelog.ILoggerStatus|null} [status] ToggleLoggerResponse status
         */

        /**
         * Constructs a new ToggleLoggerResponse.
         * @memberof lifelog
         * @classdesc Represents a ToggleLoggerResponse.
         * @implements IToggleLoggerResponse
         * @constructor
         * @param {lifelog.IToggleLoggerResponse=} [properties] Properties to set
         */
        function ToggleLoggerResponse(properties) {
            if (properties)
                for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                    if (properties[keys[i]] != null)
                        this[keys[i]] = properties[keys[i]];
        }

        /**
         * ToggleLoggerResponse success.
         * @member {boolean} success
         * @memberof lifelog.ToggleLoggerResponse
         * @instance
         */
        ToggleLoggerResponse.prototype.success = false;

        /**
         * ToggleLoggerResponse errorMessage.
         * @member {string} errorMessage
         * @memberof lifelog.ToggleLoggerResponse
         * @instance
         */
        ToggleLoggerResponse.prototype.errorMessage = "";

        /**
         * ToggleLoggerResponse status.
         * @member {lifelog.ILoggerStatus|null|undefined} status
         * @memberof lifelog.ToggleLoggerResponse
         * @instance
         */
        ToggleLoggerResponse.prototype.status = null;

        /**
         * Creates a new ToggleLoggerResponse instance using the specified properties.
         * @function create
         * @memberof lifelog.ToggleLoggerResponse
         * @static
         * @param {lifelog.IToggleLoggerResponse=} [properties] Properties to set
         * @returns {lifelog.ToggleLoggerResponse} ToggleLoggerResponse instance
         */
        ToggleLoggerResponse.create = function create(properties) {
            return new ToggleLoggerResponse(properties);
        };

        /**
         * Encodes the specified ToggleLoggerResponse message. Does not implicitly {@link lifelog.ToggleLoggerResponse.verify|verify} messages.
         * @function encode
         * @memberof lifelog.ToggleLoggerResponse
         * @static
         * @param {lifelog.IToggleLoggerResponse} message ToggleLoggerResponse message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        ToggleLoggerResponse.encode = function encode(message, writer) {
            if (!writer)
                writer = $Writer.create();
            if (message.success != null && Object.hasOwnProperty.call(message, "success"))
                writer.uint32(/* id 1, wireType 0 =*/8).bool(message.success);
            if (message.errorMessage != null && Object.hasOwnProperty.call(message, "errorMessage"))
                writer.uint32(/* id 2, wireType 2 =*/18).string(message.errorMessage);
            if (message.status != null && Object.hasOwnProperty.call(message, "status"))
                $root.lifelog.LoggerStatus.encode(message.status, writer.uint32(/* id 3, wireType 2 =*/26).fork()).ldelim();
            return writer;
        };

        /**
         * Encodes the specified ToggleLoggerResponse message, length delimited. Does not implicitly {@link lifelog.ToggleLoggerResponse.verify|verify} messages.
         * @function encodeDelimited
         * @memberof lifelog.ToggleLoggerResponse
         * @static
         * @param {lifelog.IToggleLoggerResponse} message ToggleLoggerResponse message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        ToggleLoggerResponse.encodeDelimited = function encodeDelimited(message, writer) {
            return this.encode(message, writer).ldelim();
        };

        /**
         * Decodes a ToggleLoggerResponse message from the specified reader or buffer.
         * @function decode
         * @memberof lifelog.ToggleLoggerResponse
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @param {number} [length] Message length if known beforehand
         * @returns {lifelog.ToggleLoggerResponse} ToggleLoggerResponse
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        ToggleLoggerResponse.decode = function decode(reader, length, error) {
            if (!(reader instanceof $Reader))
                reader = $Reader.create(reader);
            let end = length === undefined ? reader.len : reader.pos + length, message = new $root.lifelog.ToggleLoggerResponse();
            while (reader.pos < end) {
                let tag = reader.uint32();
                if (tag === error)
                    break;
                switch (tag >>> 3) {
                case 1: {
                        message.success = reader.bool();
                        break;
                    }
                case 2: {
                        message.errorMessage = reader.string();
                        break;
                    }
                case 3: {
                        message.status = $root.lifelog.LoggerStatus.decode(reader, reader.uint32());
                        break;
                    }
                default:
                    reader.skipType(tag & 7);
                    break;
                }
            }
            return message;
        };

        /**
         * Decodes a ToggleLoggerResponse message from the specified reader or buffer, length delimited.
         * @function decodeDelimited
         * @memberof lifelog.ToggleLoggerResponse
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @returns {lifelog.ToggleLoggerResponse} ToggleLoggerResponse
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        ToggleLoggerResponse.decodeDelimited = function decodeDelimited(reader) {
            if (!(reader instanceof $Reader))
                reader = new $Reader(reader);
            return this.decode(reader, reader.uint32());
        };

        /**
         * Verifies a ToggleLoggerResponse message.
         * @function verify
         * @memberof lifelog.ToggleLoggerResponse
         * @static
         * @param {Object.<string,*>} message Plain object to verify
         * @returns {string|null} `null` if valid, otherwise the reason why it is not
         */
        ToggleLoggerResponse.verify = function verify(message) {
            if (typeof message !== "object" || message === null)
                return "object expected";
            if (message.success != null && message.hasOwnProperty("success"))
                if (typeof message.success !== "boolean")
                    return "success: boolean expected";
            if (message.errorMessage != null && message.hasOwnProperty("errorMessage"))
                if (!$util.isString(message.errorMessage))
                    return "errorMessage: string expected";
            if (message.status != null && message.hasOwnProperty("status")) {
                let error = $root.lifelog.LoggerStatus.verify(message.status);
                if (error)
                    return "status." + error;
            }
            return null;
        };

        /**
         * Creates a ToggleLoggerResponse message from a plain object. Also converts values to their respective internal types.
         * @function fromObject
         * @memberof lifelog.ToggleLoggerResponse
         * @static
         * @param {Object.<string,*>} object Plain object
         * @returns {lifelog.ToggleLoggerResponse} ToggleLoggerResponse
         */
        ToggleLoggerResponse.fromObject = function fromObject(object) {
            if (object instanceof $root.lifelog.ToggleLoggerResponse)
                return object;
            let message = new $root.lifelog.ToggleLoggerResponse();
            if (object.success != null)
                message.success = Boolean(object.success);
            if (object.errorMessage != null)
                message.errorMessage = String(object.errorMessage);
            if (object.status != null) {
                if (typeof object.status !== "object")
                    throw TypeError(".lifelog.ToggleLoggerResponse.status: object expected");
                message.status = $root.lifelog.LoggerStatus.fromObject(object.status);
            }
            return message;
        };

        /**
         * Creates a plain object from a ToggleLoggerResponse message. Also converts values to other types if specified.
         * @function toObject
         * @memberof lifelog.ToggleLoggerResponse
         * @static
         * @param {lifelog.ToggleLoggerResponse} message ToggleLoggerResponse
         * @param {$protobuf.IConversionOptions} [options] Conversion options
         * @returns {Object.<string,*>} Plain object
         */
        ToggleLoggerResponse.toObject = function toObject(message, options) {
            if (!options)
                options = {};
            let object = {};
            if (options.defaults) {
                object.success = false;
                object.errorMessage = "";
                object.status = null;
            }
            if (message.success != null && message.hasOwnProperty("success"))
                object.success = message.success;
            if (message.errorMessage != null && message.hasOwnProperty("errorMessage"))
                object.errorMessage = message.errorMessage;
            if (message.status != null && message.hasOwnProperty("status"))
                object.status = $root.lifelog.LoggerStatus.toObject(message.status, options);
            return object;
        };

        /**
         * Converts this ToggleLoggerResponse to JSON.
         * @function toJSON
         * @memberof lifelog.ToggleLoggerResponse
         * @instance
         * @returns {Object.<string,*>} JSON object
         */
        ToggleLoggerResponse.prototype.toJSON = function toJSON() {
            return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
        };

        /**
         * Gets the default type url for ToggleLoggerResponse
         * @function getTypeUrl
         * @memberof lifelog.ToggleLoggerResponse
         * @static
         * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns {string} The default type url
         */
        ToggleLoggerResponse.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
            if (typeUrlPrefix === undefined) {
                typeUrlPrefix = "type.googleapis.com";
            }
            return typeUrlPrefix + "/lifelog.ToggleLoggerResponse";
        };

        return ToggleLoggerResponse;
    })();

    lifelog.SnapshotRequest = (function() {

        /**
         * Properties of a SnapshotRequest.
         * @memberof lifelog
         * @interface ISnapshotRequest
         * @property {Array.<string>|null} [loggers] SnapshotRequest loggers
         * @property {Object.<string,string>|null} [options] SnapshotRequest options
         */

        /**
         * Constructs a new SnapshotRequest.
         * @memberof lifelog
         * @classdesc Represents a SnapshotRequest.
         * @implements ISnapshotRequest
         * @constructor
         * @param {lifelog.ISnapshotRequest=} [properties] Properties to set
         */
        function SnapshotRequest(properties) {
            this.loggers = [];
            this.options = {};
            if (properties)
                for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                    if (properties[keys[i]] != null)
                        this[keys[i]] = properties[keys[i]];
        }

        /**
         * SnapshotRequest loggers.
         * @member {Array.<string>} loggers
         * @memberof lifelog.SnapshotRequest
         * @instance
         */
        SnapshotRequest.prototype.loggers = $util.emptyArray;

        /**
         * SnapshotRequest options.
         * @member {Object.<string,string>} options
         * @memberof lifelog.SnapshotRequest
         * @instance
         */
        SnapshotRequest.prototype.options = $util.emptyObject;

        /**
         * Creates a new SnapshotRequest instance using the specified properties.
         * @function create
         * @memberof lifelog.SnapshotRequest
         * @static
         * @param {lifelog.ISnapshotRequest=} [properties] Properties to set
         * @returns {lifelog.SnapshotRequest} SnapshotRequest instance
         */
        SnapshotRequest.create = function create(properties) {
            return new SnapshotRequest(properties);
        };

        /**
         * Encodes the specified SnapshotRequest message. Does not implicitly {@link lifelog.SnapshotRequest.verify|verify} messages.
         * @function encode
         * @memberof lifelog.SnapshotRequest
         * @static
         * @param {lifelog.ISnapshotRequest} message SnapshotRequest message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        SnapshotRequest.encode = function encode(message, writer) {
            if (!writer)
                writer = $Writer.create();
            if (message.loggers != null && message.loggers.length)
                for (let i = 0; i < message.loggers.length; ++i)
                    writer.uint32(/* id 1, wireType 2 =*/10).string(message.loggers[i]);
            if (message.options != null && Object.hasOwnProperty.call(message, "options"))
                for (let keys = Object.keys(message.options), i = 0; i < keys.length; ++i)
                    writer.uint32(/* id 2, wireType 2 =*/18).fork().uint32(/* id 1, wireType 2 =*/10).string(keys[i]).uint32(/* id 2, wireType 2 =*/18).string(message.options[keys[i]]).ldelim();
            return writer;
        };

        /**
         * Encodes the specified SnapshotRequest message, length delimited. Does not implicitly {@link lifelog.SnapshotRequest.verify|verify} messages.
         * @function encodeDelimited
         * @memberof lifelog.SnapshotRequest
         * @static
         * @param {lifelog.ISnapshotRequest} message SnapshotRequest message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        SnapshotRequest.encodeDelimited = function encodeDelimited(message, writer) {
            return this.encode(message, writer).ldelim();
        };

        /**
         * Decodes a SnapshotRequest message from the specified reader or buffer.
         * @function decode
         * @memberof lifelog.SnapshotRequest
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @param {number} [length] Message length if known beforehand
         * @returns {lifelog.SnapshotRequest} SnapshotRequest
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        SnapshotRequest.decode = function decode(reader, length, error) {
            if (!(reader instanceof $Reader))
                reader = $Reader.create(reader);
            let end = length === undefined ? reader.len : reader.pos + length, message = new $root.lifelog.SnapshotRequest(), key, value;
            while (reader.pos < end) {
                let tag = reader.uint32();
                if (tag === error)
                    break;
                switch (tag >>> 3) {
                case 1: {
                        if (!(message.loggers && message.loggers.length))
                            message.loggers = [];
                        message.loggers.push(reader.string());
                        break;
                    }
                case 2: {
                        if (message.options === $util.emptyObject)
                            message.options = {};
                        let end2 = reader.uint32() + reader.pos;
                        key = "";
                        value = "";
                        while (reader.pos < end2) {
                            let tag2 = reader.uint32();
                            switch (tag2 >>> 3) {
                            case 1:
                                key = reader.string();
                                break;
                            case 2:
                                value = reader.string();
                                break;
                            default:
                                reader.skipType(tag2 & 7);
                                break;
                            }
                        }
                        message.options[key] = value;
                        break;
                    }
                default:
                    reader.skipType(tag & 7);
                    break;
                }
            }
            return message;
        };

        /**
         * Decodes a SnapshotRequest message from the specified reader or buffer, length delimited.
         * @function decodeDelimited
         * @memberof lifelog.SnapshotRequest
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @returns {lifelog.SnapshotRequest} SnapshotRequest
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        SnapshotRequest.decodeDelimited = function decodeDelimited(reader) {
            if (!(reader instanceof $Reader))
                reader = new $Reader(reader);
            return this.decode(reader, reader.uint32());
        };

        /**
         * Verifies a SnapshotRequest message.
         * @function verify
         * @memberof lifelog.SnapshotRequest
         * @static
         * @param {Object.<string,*>} message Plain object to verify
         * @returns {string|null} `null` if valid, otherwise the reason why it is not
         */
        SnapshotRequest.verify = function verify(message) {
            if (typeof message !== "object" || message === null)
                return "object expected";
            if (message.loggers != null && message.hasOwnProperty("loggers")) {
                if (!Array.isArray(message.loggers))
                    return "loggers: array expected";
                for (let i = 0; i < message.loggers.length; ++i)
                    if (!$util.isString(message.loggers[i]))
                        return "loggers: string[] expected";
            }
            if (message.options != null && message.hasOwnProperty("options")) {
                if (!$util.isObject(message.options))
                    return "options: object expected";
                let key = Object.keys(message.options);
                for (let i = 0; i < key.length; ++i)
                    if (!$util.isString(message.options[key[i]]))
                        return "options: string{k:string} expected";
            }
            return null;
        };

        /**
         * Creates a SnapshotRequest message from a plain object. Also converts values to their respective internal types.
         * @function fromObject
         * @memberof lifelog.SnapshotRequest
         * @static
         * @param {Object.<string,*>} object Plain object
         * @returns {lifelog.SnapshotRequest} SnapshotRequest
         */
        SnapshotRequest.fromObject = function fromObject(object) {
            if (object instanceof $root.lifelog.SnapshotRequest)
                return object;
            let message = new $root.lifelog.SnapshotRequest();
            if (object.loggers) {
                if (!Array.isArray(object.loggers))
                    throw TypeError(".lifelog.SnapshotRequest.loggers: array expected");
                message.loggers = [];
                for (let i = 0; i < object.loggers.length; ++i)
                    message.loggers[i] = String(object.loggers[i]);
            }
            if (object.options) {
                if (typeof object.options !== "object")
                    throw TypeError(".lifelog.SnapshotRequest.options: object expected");
                message.options = {};
                for (let keys = Object.keys(object.options), i = 0; i < keys.length; ++i)
                    message.options[keys[i]] = String(object.options[keys[i]]);
            }
            return message;
        };

        /**
         * Creates a plain object from a SnapshotRequest message. Also converts values to other types if specified.
         * @function toObject
         * @memberof lifelog.SnapshotRequest
         * @static
         * @param {lifelog.SnapshotRequest} message SnapshotRequest
         * @param {$protobuf.IConversionOptions} [options] Conversion options
         * @returns {Object.<string,*>} Plain object
         */
        SnapshotRequest.toObject = function toObject(message, options) {
            if (!options)
                options = {};
            let object = {};
            if (options.arrays || options.defaults)
                object.loggers = [];
            if (options.objects || options.defaults)
                object.options = {};
            if (message.loggers && message.loggers.length) {
                object.loggers = [];
                for (let j = 0; j < message.loggers.length; ++j)
                    object.loggers[j] = message.loggers[j];
            }
            let keys2;
            if (message.options && (keys2 = Object.keys(message.options)).length) {
                object.options = {};
                for (let j = 0; j < keys2.length; ++j)
                    object.options[keys2[j]] = message.options[keys2[j]];
            }
            return object;
        };

        /**
         * Converts this SnapshotRequest to JSON.
         * @function toJSON
         * @memberof lifelog.SnapshotRequest
         * @instance
         * @returns {Object.<string,*>} JSON object
         */
        SnapshotRequest.prototype.toJSON = function toJSON() {
            return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
        };

        /**
         * Gets the default type url for SnapshotRequest
         * @function getTypeUrl
         * @memberof lifelog.SnapshotRequest
         * @static
         * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns {string} The default type url
         */
        SnapshotRequest.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
            if (typeUrlPrefix === undefined) {
                typeUrlPrefix = "type.googleapis.com";
            }
            return typeUrlPrefix + "/lifelog.SnapshotRequest";
        };

        return SnapshotRequest;
    })();

    lifelog.SnapshotResponse = (function() {

        /**
         * Properties of a SnapshotResponse.
         * @memberof lifelog
         * @interface ISnapshotResponse
         * @property {string|null} [snapshotId] SnapshotResponse snapshotId
         * @property {boolean|null} [success] SnapshotResponse success
         * @property {string|null} [errorMessage] SnapshotResponse errorMessage
         * @property {Array.<string>|null} [triggeredLoggers] SnapshotResponse triggeredLoggers
         */

        /**
         * Constructs a new SnapshotResponse.
         * @memberof lifelog
         * @classdesc Represents a SnapshotResponse.
         * @implements ISnapshotResponse
         * @constructor
         * @param {lifelog.ISnapshotResponse=} [properties] Properties to set
         */
        function SnapshotResponse(properties) {
            this.triggeredLoggers = [];
            if (properties)
                for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                    if (properties[keys[i]] != null)
                        this[keys[i]] = properties[keys[i]];
        }

        /**
         * SnapshotResponse snapshotId.
         * @member {string} snapshotId
         * @memberof lifelog.SnapshotResponse
         * @instance
         */
        SnapshotResponse.prototype.snapshotId = "";

        /**
         * SnapshotResponse success.
         * @member {boolean} success
         * @memberof lifelog.SnapshotResponse
         * @instance
         */
        SnapshotResponse.prototype.success = false;

        /**
         * SnapshotResponse errorMessage.
         * @member {string} errorMessage
         * @memberof lifelog.SnapshotResponse
         * @instance
         */
        SnapshotResponse.prototype.errorMessage = "";

        /**
         * SnapshotResponse triggeredLoggers.
         * @member {Array.<string>} triggeredLoggers
         * @memberof lifelog.SnapshotResponse
         * @instance
         */
        SnapshotResponse.prototype.triggeredLoggers = $util.emptyArray;

        /**
         * Creates a new SnapshotResponse instance using the specified properties.
         * @function create
         * @memberof lifelog.SnapshotResponse
         * @static
         * @param {lifelog.ISnapshotResponse=} [properties] Properties to set
         * @returns {lifelog.SnapshotResponse} SnapshotResponse instance
         */
        SnapshotResponse.create = function create(properties) {
            return new SnapshotResponse(properties);
        };

        /**
         * Encodes the specified SnapshotResponse message. Does not implicitly {@link lifelog.SnapshotResponse.verify|verify} messages.
         * @function encode
         * @memberof lifelog.SnapshotResponse
         * @static
         * @param {lifelog.ISnapshotResponse} message SnapshotResponse message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        SnapshotResponse.encode = function encode(message, writer) {
            if (!writer)
                writer = $Writer.create();
            if (message.snapshotId != null && Object.hasOwnProperty.call(message, "snapshotId"))
                writer.uint32(/* id 1, wireType 2 =*/10).string(message.snapshotId);
            if (message.success != null && Object.hasOwnProperty.call(message, "success"))
                writer.uint32(/* id 2, wireType 0 =*/16).bool(message.success);
            if (message.errorMessage != null && Object.hasOwnProperty.call(message, "errorMessage"))
                writer.uint32(/* id 3, wireType 2 =*/26).string(message.errorMessage);
            if (message.triggeredLoggers != null && message.triggeredLoggers.length)
                for (let i = 0; i < message.triggeredLoggers.length; ++i)
                    writer.uint32(/* id 4, wireType 2 =*/34).string(message.triggeredLoggers[i]);
            return writer;
        };

        /**
         * Encodes the specified SnapshotResponse message, length delimited. Does not implicitly {@link lifelog.SnapshotResponse.verify|verify} messages.
         * @function encodeDelimited
         * @memberof lifelog.SnapshotResponse
         * @static
         * @param {lifelog.ISnapshotResponse} message SnapshotResponse message or plain object to encode
         * @param {$protobuf.Writer} [writer] Writer to encode to
         * @returns {$protobuf.Writer} Writer
         */
        SnapshotResponse.encodeDelimited = function encodeDelimited(message, writer) {
            return this.encode(message, writer).ldelim();
        };

        /**
         * Decodes a SnapshotResponse message from the specified reader or buffer.
         * @function decode
         * @memberof lifelog.SnapshotResponse
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @param {number} [length] Message length if known beforehand
         * @returns {lifelog.SnapshotResponse} SnapshotResponse
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        SnapshotResponse.decode = function decode(reader, length, error) {
            if (!(reader instanceof $Reader))
                reader = $Reader.create(reader);
            let end = length === undefined ? reader.len : reader.pos + length, message = new $root.lifelog.SnapshotResponse();
            while (reader.pos < end) {
                let tag = reader.uint32();
                if (tag === error)
                    break;
                switch (tag >>> 3) {
                case 1: {
                        message.snapshotId = reader.string();
                        break;
                    }
                case 2: {
                        message.success = reader.bool();
                        break;
                    }
                case 3: {
                        message.errorMessage = reader.string();
                        break;
                    }
                case 4: {
                        if (!(message.triggeredLoggers && message.triggeredLoggers.length))
                            message.triggeredLoggers = [];
                        message.triggeredLoggers.push(reader.string());
                        break;
                    }
                default:
                    reader.skipType(tag & 7);
                    break;
                }
            }
            return message;
        };

        /**
         * Decodes a SnapshotResponse message from the specified reader or buffer, length delimited.
         * @function decodeDelimited
         * @memberof lifelog.SnapshotResponse
         * @static
         * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
         * @returns {lifelog.SnapshotResponse} SnapshotResponse
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        SnapshotResponse.decodeDelimited = function decodeDelimited(reader) {
            if (!(reader instanceof $Reader))
                reader = new $Reader(reader);
            return this.decode(reader, reader.uint32());
        };

        /**
         * Verifies a SnapshotResponse message.
         * @function verify
         * @memberof lifelog.SnapshotResponse
         * @static
         * @param {Object.<string,*>} message Plain object to verify
         * @returns {string|null} `null` if valid, otherwise the reason why it is not
         */
        SnapshotResponse.verify = function verify(message) {
            if (typeof message !== "object" || message === null)
                return "object expected";
            if (message.snapshotId != null && message.hasOwnProperty("snapshotId"))
                if (!$util.isString(message.snapshotId))
                    return "snapshotId: string expected";
            if (message.success != null && message.hasOwnProperty("success"))
                if (typeof message.success !== "boolean")
                    return "success: boolean expected";
            if (message.errorMessage != null && message.hasOwnProperty("errorMessage"))
                if (!$util.isString(message.errorMessage))
                    return "errorMessage: string expected";
            if (message.triggeredLoggers != null && message.hasOwnProperty("triggeredLoggers")) {
                if (!Array.isArray(message.triggeredLoggers))
                    return "triggeredLoggers: array expected";
                for (let i = 0; i < message.triggeredLoggers.length; ++i)
                    if (!$util.isString(message.triggeredLoggers[i]))
                        return "triggeredLoggers: string[] expected";
            }
            return null;
        };

        /**
         * Creates a SnapshotResponse message from a plain object. Also converts values to their respective internal types.
         * @function fromObject
         * @memberof lifelog.SnapshotResponse
         * @static
         * @param {Object.<string,*>} object Plain object
         * @returns {lifelog.SnapshotResponse} SnapshotResponse
         */
        SnapshotResponse.fromObject = function fromObject(object) {
            if (object instanceof $root.lifelog.SnapshotResponse)
                return object;
            let message = new $root.lifelog.SnapshotResponse();
            if (object.snapshotId != null)
                message.snapshotId = String(object.snapshotId);
            if (object.success != null)
                message.success = Boolean(object.success);
            if (object.errorMessage != null)
                message.errorMessage = String(object.errorMessage);
            if (object.triggeredLoggers) {
                if (!Array.isArray(object.triggeredLoggers))
                    throw TypeError(".lifelog.SnapshotResponse.triggeredLoggers: array expected");
                message.triggeredLoggers = [];
                for (let i = 0; i < object.triggeredLoggers.length; ++i)
                    message.triggeredLoggers[i] = String(object.triggeredLoggers[i]);
            }
            return message;
        };

        /**
         * Creates a plain object from a SnapshotResponse message. Also converts values to other types if specified.
         * @function toObject
         * @memberof lifelog.SnapshotResponse
         * @static
         * @param {lifelog.SnapshotResponse} message SnapshotResponse
         * @param {$protobuf.IConversionOptions} [options] Conversion options
         * @returns {Object.<string,*>} Plain object
         */
        SnapshotResponse.toObject = function toObject(message, options) {
            if (!options)
                options = {};
            let object = {};
            if (options.arrays || options.defaults)
                object.triggeredLoggers = [];
            if (options.defaults) {
                object.snapshotId = "";
                object.success = false;
                object.errorMessage = "";
            }
            if (message.snapshotId != null && message.hasOwnProperty("snapshotId"))
                object.snapshotId = message.snapshotId;
            if (message.success != null && message.hasOwnProperty("success"))
                object.success = message.success;
            if (message.errorMessage != null && message.hasOwnProperty("errorMessage"))
                object.errorMessage = message.errorMessage;
            if (message.triggeredLoggers && message.triggeredLoggers.length) {
                object.triggeredLoggers = [];
                for (let j = 0; j < message.triggeredLoggers.length; ++j)
                    object.triggeredLoggers[j] = message.triggeredLoggers[j];
            }
            return object;
        };

        /**
         * Converts this SnapshotResponse to JSON.
         * @function toJSON
         * @memberof lifelog.SnapshotResponse
         * @instance
         * @returns {Object.<string,*>} JSON object
         */
        SnapshotResponse.prototype.toJSON = function toJSON() {
            return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
        };

        /**
         * Gets the default type url for SnapshotResponse
         * @function getTypeUrl
         * @memberof lifelog.SnapshotResponse
         * @static
         * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns {string} The default type url
         */
        SnapshotResponse.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
            if (typeUrlPrefix === undefined) {
                typeUrlPrefix = "type.googleapis.com";
            }
            return typeUrlPrefix + "/lifelog.SnapshotResponse";
        };

        return SnapshotResponse;
    })();

    return lifelog;
})();

export { $root as default };
