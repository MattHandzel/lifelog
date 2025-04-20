import * as $protobuf from "protobufjs";
import Long = require("long");
/** Namespace lifelog. */
export namespace lifelog {

    /** Represents a LifelogService */
    class LifelogService extends $protobuf.rpc.Service {

        /**
         * Constructs a new LifelogService service.
         * @param rpcImpl RPC implementation
         * @param [requestDelimited=false] Whether requests are length-delimited
         * @param [responseDelimited=false] Whether responses are length-delimited
         */
        constructor(rpcImpl: $protobuf.RPCImpl, requestDelimited?: boolean, responseDelimited?: boolean);

        /**
         * Creates new LifelogService service using the specified rpc implementation.
         * @param rpcImpl RPC implementation
         * @param [requestDelimited=false] Whether requests are length-delimited
         * @param [responseDelimited=false] Whether responses are length-delimited
         * @returns RPC service. Useful where requests and/or responses are streamed.
         */
        public static create(rpcImpl: $protobuf.RPCImpl, requestDelimited?: boolean, responseDelimited?: boolean): LifelogService;

        /**
         * Calls Search.
         * @param request SearchRequest message or plain object
         * @param callback Node-style callback called with the error, if any, and SearchResponse
         */
        public search(request: lifelog.ISearchRequest, callback: lifelog.LifelogService.SearchCallback): void;

        /**
         * Calls Search.
         * @param request SearchRequest message or plain object
         * @returns Promise
         */
        public search(request: lifelog.ISearchRequest): Promise<lifelog.SearchResponse>;

        /**
         * Calls GetScreenshots.
         * @param request TimeRangeRequest message or plain object
         * @param callback Node-style callback called with the error, if any, and ScreenshotData
         */
        public getScreenshots(request: lifelog.ITimeRangeRequest, callback: lifelog.LifelogService.GetScreenshotsCallback): void;

        /**
         * Calls GetScreenshots.
         * @param request TimeRangeRequest message or plain object
         * @returns Promise
         */
        public getScreenshots(request: lifelog.ITimeRangeRequest): Promise<lifelog.ScreenshotData>;

        /**
         * Calls GetProcesses.
         * @param request TimeRangeRequest message or plain object
         * @param callback Node-style callback called with the error, if any, and ProcessData
         */
        public getProcesses(request: lifelog.ITimeRangeRequest, callback: lifelog.LifelogService.GetProcessesCallback): void;

        /**
         * Calls GetProcesses.
         * @param request TimeRangeRequest message or plain object
         * @returns Promise
         */
        public getProcesses(request: lifelog.ITimeRangeRequest): Promise<lifelog.ProcessData>;

        /**
         * Calls GetCameraFrames.
         * @param request TimeRangeRequest message or plain object
         * @param callback Node-style callback called with the error, if any, and CameraFrameData
         */
        public getCameraFrames(request: lifelog.ITimeRangeRequest, callback: lifelog.LifelogService.GetCameraFramesCallback): void;

        /**
         * Calls GetCameraFrames.
         * @param request TimeRangeRequest message or plain object
         * @returns Promise
         */
        public getCameraFrames(request: lifelog.ITimeRangeRequest): Promise<lifelog.CameraFrameData>;

        /**
         * Calls GetActivitySummary.
         * @param request TimeRangeRequest message or plain object
         * @param callback Node-style callback called with the error, if any, and ActivitySummary
         */
        public getActivitySummary(request: lifelog.ITimeRangeRequest, callback: lifelog.LifelogService.GetActivitySummaryCallback): void;

        /**
         * Calls GetActivitySummary.
         * @param request TimeRangeRequest message or plain object
         * @returns Promise
         */
        public getActivitySummary(request: lifelog.ITimeRangeRequest): Promise<lifelog.ActivitySummary>;

        /**
         * Calls GetProcessStats.
         * @param request ProcessStatsRequest message or plain object
         * @param callback Node-style callback called with the error, if any, and ProcessStatsResponse
         */
        public getProcessStats(request: lifelog.IProcessStatsRequest, callback: lifelog.LifelogService.GetProcessStatsCallback): void;

        /**
         * Calls GetProcessStats.
         * @param request ProcessStatsRequest message or plain object
         * @returns Promise
         */
        public getProcessStats(request: lifelog.IProcessStatsRequest): Promise<lifelog.ProcessStatsResponse>;

        /**
         * Calls Login.
         * @param request LoginRequest message or plain object
         * @param callback Node-style callback called with the error, if any, and LoginResponse
         */
        public login(request: lifelog.ILoginRequest, callback: lifelog.LifelogService.LoginCallback): void;

        /**
         * Calls Login.
         * @param request LoginRequest message or plain object
         * @returns Promise
         */
        public login(request: lifelog.ILoginRequest): Promise<lifelog.LoginResponse>;

        /**
         * Calls Register.
         * @param request RegisterRequest message or plain object
         * @param callback Node-style callback called with the error, if any, and RegisterResponse
         */
        public register(request: lifelog.IRegisterRequest, callback: lifelog.LifelogService.RegisterCallback): void;

        /**
         * Calls Register.
         * @param request RegisterRequest message or plain object
         * @returns Promise
         */
        public register(request: lifelog.IRegisterRequest): Promise<lifelog.RegisterResponse>;

        /**
         * Calls GetUserProfile.
         * @param request UserRequest message or plain object
         * @param callback Node-style callback called with the error, if any, and UserProfile
         */
        public getUserProfile(request: lifelog.IUserRequest, callback: lifelog.LifelogService.GetUserProfileCallback): void;

        /**
         * Calls GetUserProfile.
         * @param request UserRequest message or plain object
         * @returns Promise
         */
        public getUserProfile(request: lifelog.IUserRequest): Promise<lifelog.UserProfile>;

        /**
         * Calls GetLoggerStatus.
         * @param request LoggerStatusRequest message or plain object
         * @param callback Node-style callback called with the error, if any, and LoggerStatusResponse
         */
        public getLoggerStatus(request: lifelog.ILoggerStatusRequest, callback: lifelog.LifelogService.GetLoggerStatusCallback): void;

        /**
         * Calls GetLoggerStatus.
         * @param request LoggerStatusRequest message or plain object
         * @returns Promise
         */
        public getLoggerStatus(request: lifelog.ILoggerStatusRequest): Promise<lifelog.LoggerStatusResponse>;

        /**
         * Calls ToggleLogger.
         * @param request ToggleLoggerRequest message or plain object
         * @param callback Node-style callback called with the error, if any, and ToggleLoggerResponse
         */
        public toggleLogger(request: lifelog.IToggleLoggerRequest, callback: lifelog.LifelogService.ToggleLoggerCallback): void;

        /**
         * Calls ToggleLogger.
         * @param request ToggleLoggerRequest message or plain object
         * @returns Promise
         */
        public toggleLogger(request: lifelog.IToggleLoggerRequest): Promise<lifelog.ToggleLoggerResponse>;

        /**
         * Calls TakeSnapshot.
         * @param request SnapshotRequest message or plain object
         * @param callback Node-style callback called with the error, if any, and SnapshotResponse
         */
        public takeSnapshot(request: lifelog.ISnapshotRequest, callback: lifelog.LifelogService.TakeSnapshotCallback): void;

        /**
         * Calls TakeSnapshot.
         * @param request SnapshotRequest message or plain object
         * @returns Promise
         */
        public takeSnapshot(request: lifelog.ISnapshotRequest): Promise<lifelog.SnapshotResponse>;
    }

    namespace LifelogService {

        /**
         * Callback as used by {@link lifelog.LifelogService#search}.
         * @param error Error, if any
         * @param [response] SearchResponse
         */
        type SearchCallback = (error: (Error|null), response?: lifelog.SearchResponse) => void;

        /**
         * Callback as used by {@link lifelog.LifelogService#getScreenshots}.
         * @param error Error, if any
         * @param [response] ScreenshotData
         */
        type GetScreenshotsCallback = (error: (Error|null), response?: lifelog.ScreenshotData) => void;

        /**
         * Callback as used by {@link lifelog.LifelogService#getProcesses}.
         * @param error Error, if any
         * @param [response] ProcessData
         */
        type GetProcessesCallback = (error: (Error|null), response?: lifelog.ProcessData) => void;

        /**
         * Callback as used by {@link lifelog.LifelogService#getCameraFrames}.
         * @param error Error, if any
         * @param [response] CameraFrameData
         */
        type GetCameraFramesCallback = (error: (Error|null), response?: lifelog.CameraFrameData) => void;

        /**
         * Callback as used by {@link lifelog.LifelogService#getActivitySummary}.
         * @param error Error, if any
         * @param [response] ActivitySummary
         */
        type GetActivitySummaryCallback = (error: (Error|null), response?: lifelog.ActivitySummary) => void;

        /**
         * Callback as used by {@link lifelog.LifelogService#getProcessStats}.
         * @param error Error, if any
         * @param [response] ProcessStatsResponse
         */
        type GetProcessStatsCallback = (error: (Error|null), response?: lifelog.ProcessStatsResponse) => void;

        /**
         * Callback as used by {@link lifelog.LifelogService#login}.
         * @param error Error, if any
         * @param [response] LoginResponse
         */
        type LoginCallback = (error: (Error|null), response?: lifelog.LoginResponse) => void;

        /**
         * Callback as used by {@link lifelog.LifelogService#register}.
         * @param error Error, if any
         * @param [response] RegisterResponse
         */
        type RegisterCallback = (error: (Error|null), response?: lifelog.RegisterResponse) => void;

        /**
         * Callback as used by {@link lifelog.LifelogService#getUserProfile}.
         * @param error Error, if any
         * @param [response] UserProfile
         */
        type GetUserProfileCallback = (error: (Error|null), response?: lifelog.UserProfile) => void;

        /**
         * Callback as used by {@link lifelog.LifelogService#getLoggerStatus}.
         * @param error Error, if any
         * @param [response] LoggerStatusResponse
         */
        type GetLoggerStatusCallback = (error: (Error|null), response?: lifelog.LoggerStatusResponse) => void;

        /**
         * Callback as used by {@link lifelog.LifelogService#toggleLogger}.
         * @param error Error, if any
         * @param [response] ToggleLoggerResponse
         */
        type ToggleLoggerCallback = (error: (Error|null), response?: lifelog.ToggleLoggerResponse) => void;

        /**
         * Callback as used by {@link lifelog.LifelogService#takeSnapshot}.
         * @param error Error, if any
         * @param [response] SnapshotResponse
         */
        type TakeSnapshotCallback = (error: (Error|null), response?: lifelog.SnapshotResponse) => void;
    }

    /** Properties of a TimeRangeRequest. */
    interface ITimeRangeRequest {

        /** TimeRangeRequest startTime */
        startTime?: (string|null);

        /** TimeRangeRequest endTime */
        endTime?: (string|null);

        /** TimeRangeRequest limit */
        limit?: (number|null);

        /** TimeRangeRequest offset */
        offset?: (number|null);
    }

    /** Represents a TimeRangeRequest. */
    class TimeRangeRequest implements ITimeRangeRequest {

        /**
         * Constructs a new TimeRangeRequest.
         * @param [properties] Properties to set
         */
        constructor(properties?: lifelog.ITimeRangeRequest);

        /** TimeRangeRequest startTime. */
        public startTime: string;

        /** TimeRangeRequest endTime. */
        public endTime: string;

        /** TimeRangeRequest limit. */
        public limit: number;

        /** TimeRangeRequest offset. */
        public offset: number;

        /**
         * Creates a new TimeRangeRequest instance using the specified properties.
         * @param [properties] Properties to set
         * @returns TimeRangeRequest instance
         */
        public static create(properties?: lifelog.ITimeRangeRequest): lifelog.TimeRangeRequest;

        /**
         * Encodes the specified TimeRangeRequest message. Does not implicitly {@link lifelog.TimeRangeRequest.verify|verify} messages.
         * @param message TimeRangeRequest message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: lifelog.ITimeRangeRequest, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified TimeRangeRequest message, length delimited. Does not implicitly {@link lifelog.TimeRangeRequest.verify|verify} messages.
         * @param message TimeRangeRequest message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: lifelog.ITimeRangeRequest, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes a TimeRangeRequest message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns TimeRangeRequest
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): lifelog.TimeRangeRequest;

        /**
         * Decodes a TimeRangeRequest message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns TimeRangeRequest
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): lifelog.TimeRangeRequest;

        /**
         * Verifies a TimeRangeRequest message.
         * @param message Plain object to verify
         * @returns `null` if valid, otherwise the reason why it is not
         */
        public static verify(message: { [k: string]: any }): (string|null);

        /**
         * Creates a TimeRangeRequest message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns TimeRangeRequest
         */
        public static fromObject(object: { [k: string]: any }): lifelog.TimeRangeRequest;

        /**
         * Creates a plain object from a TimeRangeRequest message. Also converts values to other types if specified.
         * @param message TimeRangeRequest
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: lifelog.TimeRangeRequest, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this TimeRangeRequest to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for TimeRangeRequest
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }

    /** Properties of a TimeStamped. */
    interface ITimeStamped {

        /** TimeStamped timestamp */
        timestamp?: (string|null);
    }

    /** Represents a TimeStamped. */
    class TimeStamped implements ITimeStamped {

        /**
         * Constructs a new TimeStamped.
         * @param [properties] Properties to set
         */
        constructor(properties?: lifelog.ITimeStamped);

        /** TimeStamped timestamp. */
        public timestamp: string;

        /**
         * Creates a new TimeStamped instance using the specified properties.
         * @param [properties] Properties to set
         * @returns TimeStamped instance
         */
        public static create(properties?: lifelog.ITimeStamped): lifelog.TimeStamped;

        /**
         * Encodes the specified TimeStamped message. Does not implicitly {@link lifelog.TimeStamped.verify|verify} messages.
         * @param message TimeStamped message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: lifelog.ITimeStamped, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified TimeStamped message, length delimited. Does not implicitly {@link lifelog.TimeStamped.verify|verify} messages.
         * @param message TimeStamped message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: lifelog.ITimeStamped, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes a TimeStamped message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns TimeStamped
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): lifelog.TimeStamped;

        /**
         * Decodes a TimeStamped message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns TimeStamped
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): lifelog.TimeStamped;

        /**
         * Verifies a TimeStamped message.
         * @param message Plain object to verify
         * @returns `null` if valid, otherwise the reason why it is not
         */
        public static verify(message: { [k: string]: any }): (string|null);

        /**
         * Creates a TimeStamped message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns TimeStamped
         */
        public static fromObject(object: { [k: string]: any }): lifelog.TimeStamped;

        /**
         * Creates a plain object from a TimeStamped message. Also converts values to other types if specified.
         * @param message TimeStamped
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: lifelog.TimeStamped, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this TimeStamped to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for TimeStamped
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }

    /** Properties of a SearchRequest. */
    interface ISearchRequest {

        /** SearchRequest query */
        query?: (string|null);

        /** SearchRequest dataSources */
        dataSources?: (string[]|null);

        /** SearchRequest timeRange */
        timeRange?: (lifelog.ITimeRangeRequest|null);

        /** SearchRequest useLlm */
        useLlm?: (boolean|null);
    }

    /** Represents a SearchRequest. */
    class SearchRequest implements ISearchRequest {

        /**
         * Constructs a new SearchRequest.
         * @param [properties] Properties to set
         */
        constructor(properties?: lifelog.ISearchRequest);

        /** SearchRequest query. */
        public query: string;

        /** SearchRequest dataSources. */
        public dataSources: string[];

        /** SearchRequest timeRange. */
        public timeRange?: (lifelog.ITimeRangeRequest|null);

        /** SearchRequest useLlm. */
        public useLlm: boolean;

        /**
         * Creates a new SearchRequest instance using the specified properties.
         * @param [properties] Properties to set
         * @returns SearchRequest instance
         */
        public static create(properties?: lifelog.ISearchRequest): lifelog.SearchRequest;

        /**
         * Encodes the specified SearchRequest message. Does not implicitly {@link lifelog.SearchRequest.verify|verify} messages.
         * @param message SearchRequest message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: lifelog.ISearchRequest, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified SearchRequest message, length delimited. Does not implicitly {@link lifelog.SearchRequest.verify|verify} messages.
         * @param message SearchRequest message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: lifelog.ISearchRequest, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes a SearchRequest message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns SearchRequest
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): lifelog.SearchRequest;

        /**
         * Decodes a SearchRequest message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns SearchRequest
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): lifelog.SearchRequest;

        /**
         * Verifies a SearchRequest message.
         * @param message Plain object to verify
         * @returns `null` if valid, otherwise the reason why it is not
         */
        public static verify(message: { [k: string]: any }): (string|null);

        /**
         * Creates a SearchRequest message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns SearchRequest
         */
        public static fromObject(object: { [k: string]: any }): lifelog.SearchRequest;

        /**
         * Creates a plain object from a SearchRequest message. Also converts values to other types if specified.
         * @param message SearchRequest
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: lifelog.SearchRequest, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this SearchRequest to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for SearchRequest
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }

    /** Properties of a SearchResult. */
    interface ISearchResult {

        /** SearchResult type */
        type?: (string|null);

        /** SearchResult timestamp */
        timestamp?: (string|null);

        /** SearchResult sourceId */
        sourceId?: (string|null);

        /** SearchResult metadata */
        metadata?: ({ [k: string]: string }|null);

        /** SearchResult binaryData */
        binaryData?: (Uint8Array|null);

        /** SearchResult textData */
        textData?: (string|null);

        /** SearchResult relevanceScore */
        relevanceScore?: (number|null);
    }

    /** Represents a SearchResult. */
    class SearchResult implements ISearchResult {

        /**
         * Constructs a new SearchResult.
         * @param [properties] Properties to set
         */
        constructor(properties?: lifelog.ISearchResult);

        /** SearchResult type. */
        public type: string;

        /** SearchResult timestamp. */
        public timestamp: string;

        /** SearchResult sourceId. */
        public sourceId: string;

        /** SearchResult metadata. */
        public metadata: { [k: string]: string };

        /** SearchResult binaryData. */
        public binaryData?: (Uint8Array|null);

        /** SearchResult textData. */
        public textData?: (string|null);

        /** SearchResult relevanceScore. */
        public relevanceScore: number;

        /** SearchResult data. */
        public data?: ("binaryData"|"textData");

        /**
         * Creates a new SearchResult instance using the specified properties.
         * @param [properties] Properties to set
         * @returns SearchResult instance
         */
        public static create(properties?: lifelog.ISearchResult): lifelog.SearchResult;

        /**
         * Encodes the specified SearchResult message. Does not implicitly {@link lifelog.SearchResult.verify|verify} messages.
         * @param message SearchResult message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: lifelog.ISearchResult, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified SearchResult message, length delimited. Does not implicitly {@link lifelog.SearchResult.verify|verify} messages.
         * @param message SearchResult message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: lifelog.ISearchResult, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes a SearchResult message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns SearchResult
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): lifelog.SearchResult;

        /**
         * Decodes a SearchResult message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns SearchResult
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): lifelog.SearchResult;

        /**
         * Verifies a SearchResult message.
         * @param message Plain object to verify
         * @returns `null` if valid, otherwise the reason why it is not
         */
        public static verify(message: { [k: string]: any }): (string|null);

        /**
         * Creates a SearchResult message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns SearchResult
         */
        public static fromObject(object: { [k: string]: any }): lifelog.SearchResult;

        /**
         * Creates a plain object from a SearchResult message. Also converts values to other types if specified.
         * @param message SearchResult
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: lifelog.SearchResult, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this SearchResult to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for SearchResult
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }

    /** Properties of a SearchResponse. */
    interface ISearchResponse {

        /** SearchResponse results */
        results?: (lifelog.ISearchResult[]|null);

        /** SearchResponse totalResults */
        totalResults?: (number|null);

        /** SearchResponse searchId */
        searchId?: (string|null);
    }

    /** Represents a SearchResponse. */
    class SearchResponse implements ISearchResponse {

        /**
         * Constructs a new SearchResponse.
         * @param [properties] Properties to set
         */
        constructor(properties?: lifelog.ISearchResponse);

        /** SearchResponse results. */
        public results: lifelog.ISearchResult[];

        /** SearchResponse totalResults. */
        public totalResults: number;

        /** SearchResponse searchId. */
        public searchId: string;

        /**
         * Creates a new SearchResponse instance using the specified properties.
         * @param [properties] Properties to set
         * @returns SearchResponse instance
         */
        public static create(properties?: lifelog.ISearchResponse): lifelog.SearchResponse;

        /**
         * Encodes the specified SearchResponse message. Does not implicitly {@link lifelog.SearchResponse.verify|verify} messages.
         * @param message SearchResponse message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: lifelog.ISearchResponse, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified SearchResponse message, length delimited. Does not implicitly {@link lifelog.SearchResponse.verify|verify} messages.
         * @param message SearchResponse message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: lifelog.ISearchResponse, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes a SearchResponse message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns SearchResponse
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): lifelog.SearchResponse;

        /**
         * Decodes a SearchResponse message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns SearchResponse
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): lifelog.SearchResponse;

        /**
         * Verifies a SearchResponse message.
         * @param message Plain object to verify
         * @returns `null` if valid, otherwise the reason why it is not
         */
        public static verify(message: { [k: string]: any }): (string|null);

        /**
         * Creates a SearchResponse message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns SearchResponse
         */
        public static fromObject(object: { [k: string]: any }): lifelog.SearchResponse;

        /**
         * Creates a plain object from a SearchResponse message. Also converts values to other types if specified.
         * @param message SearchResponse
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: lifelog.SearchResponse, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this SearchResponse to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for SearchResponse
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }

    /** Properties of a ScreenshotData. */
    interface IScreenshotData {

        /** ScreenshotData id */
        id?: (string|null);

        /** ScreenshotData timestamp */
        timestamp?: (string|null);

        /** ScreenshotData imageData */
        imageData?: (Uint8Array|null);

        /** ScreenshotData mimeType */
        mimeType?: (string|null);

        /** ScreenshotData metadata */
        metadata?: ({ [k: string]: string }|null);
    }

    /** Represents a ScreenshotData. */
    class ScreenshotData implements IScreenshotData {

        /**
         * Constructs a new ScreenshotData.
         * @param [properties] Properties to set
         */
        constructor(properties?: lifelog.IScreenshotData);

        /** ScreenshotData id. */
        public id: string;

        /** ScreenshotData timestamp. */
        public timestamp: string;

        /** ScreenshotData imageData. */
        public imageData: Uint8Array;

        /** ScreenshotData mimeType. */
        public mimeType: string;

        /** ScreenshotData metadata. */
        public metadata: { [k: string]: string };

        /**
         * Creates a new ScreenshotData instance using the specified properties.
         * @param [properties] Properties to set
         * @returns ScreenshotData instance
         */
        public static create(properties?: lifelog.IScreenshotData): lifelog.ScreenshotData;

        /**
         * Encodes the specified ScreenshotData message. Does not implicitly {@link lifelog.ScreenshotData.verify|verify} messages.
         * @param message ScreenshotData message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: lifelog.IScreenshotData, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified ScreenshotData message, length delimited. Does not implicitly {@link lifelog.ScreenshotData.verify|verify} messages.
         * @param message ScreenshotData message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: lifelog.IScreenshotData, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes a ScreenshotData message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns ScreenshotData
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): lifelog.ScreenshotData;

        /**
         * Decodes a ScreenshotData message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns ScreenshotData
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): lifelog.ScreenshotData;

        /**
         * Verifies a ScreenshotData message.
         * @param message Plain object to verify
         * @returns `null` if valid, otherwise the reason why it is not
         */
        public static verify(message: { [k: string]: any }): (string|null);

        /**
         * Creates a ScreenshotData message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns ScreenshotData
         */
        public static fromObject(object: { [k: string]: any }): lifelog.ScreenshotData;

        /**
         * Creates a plain object from a ScreenshotData message. Also converts values to other types if specified.
         * @param message ScreenshotData
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: lifelog.ScreenshotData, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this ScreenshotData to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for ScreenshotData
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }

    /** Properties of a ProcessData. */
    interface IProcessData {

        /** ProcessData id */
        id?: (string|null);

        /** ProcessData timestamp */
        timestamp?: (string|null);

        /** ProcessData processName */
        processName?: (string|null);

        /** ProcessData windowTitle */
        windowTitle?: (string|null);

        /** ProcessData pid */
        pid?: (number|null);

        /** ProcessData cpuUsage */
        cpuUsage?: (number|null);

        /** ProcessData memoryUsage */
        memoryUsage?: (number|null);

        /** ProcessData isFocused */
        isFocused?: (boolean|null);
    }

    /** Represents a ProcessData. */
    class ProcessData implements IProcessData {

        /**
         * Constructs a new ProcessData.
         * @param [properties] Properties to set
         */
        constructor(properties?: lifelog.IProcessData);

        /** ProcessData id. */
        public id: string;

        /** ProcessData timestamp. */
        public timestamp: string;

        /** ProcessData processName. */
        public processName: string;

        /** ProcessData windowTitle. */
        public windowTitle: string;

        /** ProcessData pid. */
        public pid: number;

        /** ProcessData cpuUsage. */
        public cpuUsage: number;

        /** ProcessData memoryUsage. */
        public memoryUsage: number;

        /** ProcessData isFocused. */
        public isFocused: boolean;

        /**
         * Creates a new ProcessData instance using the specified properties.
         * @param [properties] Properties to set
         * @returns ProcessData instance
         */
        public static create(properties?: lifelog.IProcessData): lifelog.ProcessData;

        /**
         * Encodes the specified ProcessData message. Does not implicitly {@link lifelog.ProcessData.verify|verify} messages.
         * @param message ProcessData message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: lifelog.IProcessData, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified ProcessData message, length delimited. Does not implicitly {@link lifelog.ProcessData.verify|verify} messages.
         * @param message ProcessData message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: lifelog.IProcessData, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes a ProcessData message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns ProcessData
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): lifelog.ProcessData;

        /**
         * Decodes a ProcessData message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns ProcessData
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): lifelog.ProcessData;

        /**
         * Verifies a ProcessData message.
         * @param message Plain object to verify
         * @returns `null` if valid, otherwise the reason why it is not
         */
        public static verify(message: { [k: string]: any }): (string|null);

        /**
         * Creates a ProcessData message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns ProcessData
         */
        public static fromObject(object: { [k: string]: any }): lifelog.ProcessData;

        /**
         * Creates a plain object from a ProcessData message. Also converts values to other types if specified.
         * @param message ProcessData
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: lifelog.ProcessData, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this ProcessData to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for ProcessData
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }

    /** Properties of a ProcessStatsRequest. */
    interface IProcessStatsRequest {

        /** ProcessStatsRequest timeRange */
        timeRange?: (lifelog.ITimeRangeRequest|null);

        /** ProcessStatsRequest processName */
        processName?: (string|null);

        /** ProcessStatsRequest aggregate */
        aggregate?: (boolean|null);
    }

    /** Represents a ProcessStatsRequest. */
    class ProcessStatsRequest implements IProcessStatsRequest {

        /**
         * Constructs a new ProcessStatsRequest.
         * @param [properties] Properties to set
         */
        constructor(properties?: lifelog.IProcessStatsRequest);

        /** ProcessStatsRequest timeRange. */
        public timeRange?: (lifelog.ITimeRangeRequest|null);

        /** ProcessStatsRequest processName. */
        public processName: string;

        /** ProcessStatsRequest aggregate. */
        public aggregate: boolean;

        /**
         * Creates a new ProcessStatsRequest instance using the specified properties.
         * @param [properties] Properties to set
         * @returns ProcessStatsRequest instance
         */
        public static create(properties?: lifelog.IProcessStatsRequest): lifelog.ProcessStatsRequest;

        /**
         * Encodes the specified ProcessStatsRequest message. Does not implicitly {@link lifelog.ProcessStatsRequest.verify|verify} messages.
         * @param message ProcessStatsRequest message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: lifelog.IProcessStatsRequest, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified ProcessStatsRequest message, length delimited. Does not implicitly {@link lifelog.ProcessStatsRequest.verify|verify} messages.
         * @param message ProcessStatsRequest message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: lifelog.IProcessStatsRequest, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes a ProcessStatsRequest message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns ProcessStatsRequest
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): lifelog.ProcessStatsRequest;

        /**
         * Decodes a ProcessStatsRequest message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns ProcessStatsRequest
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): lifelog.ProcessStatsRequest;

        /**
         * Verifies a ProcessStatsRequest message.
         * @param message Plain object to verify
         * @returns `null` if valid, otherwise the reason why it is not
         */
        public static verify(message: { [k: string]: any }): (string|null);

        /**
         * Creates a ProcessStatsRequest message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns ProcessStatsRequest
         */
        public static fromObject(object: { [k: string]: any }): lifelog.ProcessStatsRequest;

        /**
         * Creates a plain object from a ProcessStatsRequest message. Also converts values to other types if specified.
         * @param message ProcessStatsRequest
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: lifelog.ProcessStatsRequest, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this ProcessStatsRequest to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for ProcessStatsRequest
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }

    /** Properties of a ProcessStatsSummary. */
    interface IProcessStatsSummary {

        /** ProcessStatsSummary processName */
        processName?: (string|null);

        /** ProcessStatsSummary totalActiveTime */
        totalActiveTime?: (number|null);

        /** ProcessStatsSummary averageCpuUsage */
        averageCpuUsage?: (number|null);

        /** ProcessStatsSummary averageMemoryUsage */
        averageMemoryUsage?: (number|null);

        /** ProcessStatsSummary focusCount */
        focusCount?: (number|null);
    }

    /** Represents a ProcessStatsSummary. */
    class ProcessStatsSummary implements IProcessStatsSummary {

        /**
         * Constructs a new ProcessStatsSummary.
         * @param [properties] Properties to set
         */
        constructor(properties?: lifelog.IProcessStatsSummary);

        /** ProcessStatsSummary processName. */
        public processName: string;

        /** ProcessStatsSummary totalActiveTime. */
        public totalActiveTime: number;

        /** ProcessStatsSummary averageCpuUsage. */
        public averageCpuUsage: number;

        /** ProcessStatsSummary averageMemoryUsage. */
        public averageMemoryUsage: number;

        /** ProcessStatsSummary focusCount. */
        public focusCount: number;

        /**
         * Creates a new ProcessStatsSummary instance using the specified properties.
         * @param [properties] Properties to set
         * @returns ProcessStatsSummary instance
         */
        public static create(properties?: lifelog.IProcessStatsSummary): lifelog.ProcessStatsSummary;

        /**
         * Encodes the specified ProcessStatsSummary message. Does not implicitly {@link lifelog.ProcessStatsSummary.verify|verify} messages.
         * @param message ProcessStatsSummary message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: lifelog.IProcessStatsSummary, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified ProcessStatsSummary message, length delimited. Does not implicitly {@link lifelog.ProcessStatsSummary.verify|verify} messages.
         * @param message ProcessStatsSummary message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: lifelog.IProcessStatsSummary, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes a ProcessStatsSummary message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns ProcessStatsSummary
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): lifelog.ProcessStatsSummary;

        /**
         * Decodes a ProcessStatsSummary message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns ProcessStatsSummary
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): lifelog.ProcessStatsSummary;

        /**
         * Verifies a ProcessStatsSummary message.
         * @param message Plain object to verify
         * @returns `null` if valid, otherwise the reason why it is not
         */
        public static verify(message: { [k: string]: any }): (string|null);

        /**
         * Creates a ProcessStatsSummary message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns ProcessStatsSummary
         */
        public static fromObject(object: { [k: string]: any }): lifelog.ProcessStatsSummary;

        /**
         * Creates a plain object from a ProcessStatsSummary message. Also converts values to other types if specified.
         * @param message ProcessStatsSummary
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: lifelog.ProcessStatsSummary, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this ProcessStatsSummary to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for ProcessStatsSummary
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }

    /** Properties of a ProcessStatsResponse. */
    interface IProcessStatsResponse {

        /** ProcessStatsResponse summaries */
        summaries?: (lifelog.IProcessStatsSummary[]|null);

        /** ProcessStatsResponse usageByHour */
        usageByHour?: ({ [k: string]: number }|null);
    }

    /** Represents a ProcessStatsResponse. */
    class ProcessStatsResponse implements IProcessStatsResponse {

        /**
         * Constructs a new ProcessStatsResponse.
         * @param [properties] Properties to set
         */
        constructor(properties?: lifelog.IProcessStatsResponse);

        /** ProcessStatsResponse summaries. */
        public summaries: lifelog.IProcessStatsSummary[];

        /** ProcessStatsResponse usageByHour. */
        public usageByHour: { [k: string]: number };

        /**
         * Creates a new ProcessStatsResponse instance using the specified properties.
         * @param [properties] Properties to set
         * @returns ProcessStatsResponse instance
         */
        public static create(properties?: lifelog.IProcessStatsResponse): lifelog.ProcessStatsResponse;

        /**
         * Encodes the specified ProcessStatsResponse message. Does not implicitly {@link lifelog.ProcessStatsResponse.verify|verify} messages.
         * @param message ProcessStatsResponse message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: lifelog.IProcessStatsResponse, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified ProcessStatsResponse message, length delimited. Does not implicitly {@link lifelog.ProcessStatsResponse.verify|verify} messages.
         * @param message ProcessStatsResponse message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: lifelog.IProcessStatsResponse, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes a ProcessStatsResponse message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns ProcessStatsResponse
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): lifelog.ProcessStatsResponse;

        /**
         * Decodes a ProcessStatsResponse message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns ProcessStatsResponse
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): lifelog.ProcessStatsResponse;

        /**
         * Verifies a ProcessStatsResponse message.
         * @param message Plain object to verify
         * @returns `null` if valid, otherwise the reason why it is not
         */
        public static verify(message: { [k: string]: any }): (string|null);

        /**
         * Creates a ProcessStatsResponse message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns ProcessStatsResponse
         */
        public static fromObject(object: { [k: string]: any }): lifelog.ProcessStatsResponse;

        /**
         * Creates a plain object from a ProcessStatsResponse message. Also converts values to other types if specified.
         * @param message ProcessStatsResponse
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: lifelog.ProcessStatsResponse, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this ProcessStatsResponse to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for ProcessStatsResponse
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }

    /** Properties of a CameraFrameData. */
    interface ICameraFrameData {

        /** CameraFrameData id */
        id?: (string|null);

        /** CameraFrameData timestamp */
        timestamp?: (string|null);

        /** CameraFrameData imageData */
        imageData?: (Uint8Array|null);

        /** CameraFrameData mimeType */
        mimeType?: (string|null);

        /** CameraFrameData metadata */
        metadata?: ({ [k: string]: string }|null);
    }

    /** Represents a CameraFrameData. */
    class CameraFrameData implements ICameraFrameData {

        /**
         * Constructs a new CameraFrameData.
         * @param [properties] Properties to set
         */
        constructor(properties?: lifelog.ICameraFrameData);

        /** CameraFrameData id. */
        public id: string;

        /** CameraFrameData timestamp. */
        public timestamp: string;

        /** CameraFrameData imageData. */
        public imageData: Uint8Array;

        /** CameraFrameData mimeType. */
        public mimeType: string;

        /** CameraFrameData metadata. */
        public metadata: { [k: string]: string };

        /**
         * Creates a new CameraFrameData instance using the specified properties.
         * @param [properties] Properties to set
         * @returns CameraFrameData instance
         */
        public static create(properties?: lifelog.ICameraFrameData): lifelog.CameraFrameData;

        /**
         * Encodes the specified CameraFrameData message. Does not implicitly {@link lifelog.CameraFrameData.verify|verify} messages.
         * @param message CameraFrameData message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: lifelog.ICameraFrameData, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified CameraFrameData message, length delimited. Does not implicitly {@link lifelog.CameraFrameData.verify|verify} messages.
         * @param message CameraFrameData message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: lifelog.ICameraFrameData, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes a CameraFrameData message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns CameraFrameData
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): lifelog.CameraFrameData;

        /**
         * Decodes a CameraFrameData message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns CameraFrameData
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): lifelog.CameraFrameData;

        /**
         * Verifies a CameraFrameData message.
         * @param message Plain object to verify
         * @returns `null` if valid, otherwise the reason why it is not
         */
        public static verify(message: { [k: string]: any }): (string|null);

        /**
         * Creates a CameraFrameData message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns CameraFrameData
         */
        public static fromObject(object: { [k: string]: any }): lifelog.CameraFrameData;

        /**
         * Creates a plain object from a CameraFrameData message. Also converts values to other types if specified.
         * @param message CameraFrameData
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: lifelog.CameraFrameData, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this CameraFrameData to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for CameraFrameData
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }

    /** Properties of an ActivitySummary. */
    interface IActivitySummary {

        /** ActivitySummary timeRange */
        timeRange?: (lifelog.ITimeRangeRequest|null);

        /** ActivitySummary activityPeriods */
        activityPeriods?: (lifelog.IActivityPeriod[]|null);

        /** ActivitySummary appUsage */
        appUsage?: ({ [k: string]: number }|null);

        /** ActivitySummary totalScreenshots */
        totalScreenshots?: (number|null);

        /** ActivitySummary totalCameraFrames */
        totalCameraFrames?: (number|null);

        /** ActivitySummary totalByLogger */
        totalByLogger?: ({ [k: string]: number }|null);
    }

    /** Represents an ActivitySummary. */
    class ActivitySummary implements IActivitySummary {

        /**
         * Constructs a new ActivitySummary.
         * @param [properties] Properties to set
         */
        constructor(properties?: lifelog.IActivitySummary);

        /** ActivitySummary timeRange. */
        public timeRange?: (lifelog.ITimeRangeRequest|null);

        /** ActivitySummary activityPeriods. */
        public activityPeriods: lifelog.IActivityPeriod[];

        /** ActivitySummary appUsage. */
        public appUsage: { [k: string]: number };

        /** ActivitySummary totalScreenshots. */
        public totalScreenshots: number;

        /** ActivitySummary totalCameraFrames. */
        public totalCameraFrames: number;

        /** ActivitySummary totalByLogger. */
        public totalByLogger: { [k: string]: number };

        /**
         * Creates a new ActivitySummary instance using the specified properties.
         * @param [properties] Properties to set
         * @returns ActivitySummary instance
         */
        public static create(properties?: lifelog.IActivitySummary): lifelog.ActivitySummary;

        /**
         * Encodes the specified ActivitySummary message. Does not implicitly {@link lifelog.ActivitySummary.verify|verify} messages.
         * @param message ActivitySummary message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: lifelog.IActivitySummary, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified ActivitySummary message, length delimited. Does not implicitly {@link lifelog.ActivitySummary.verify|verify} messages.
         * @param message ActivitySummary message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: lifelog.IActivitySummary, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes an ActivitySummary message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns ActivitySummary
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): lifelog.ActivitySummary;

        /**
         * Decodes an ActivitySummary message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns ActivitySummary
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): lifelog.ActivitySummary;

        /**
         * Verifies an ActivitySummary message.
         * @param message Plain object to verify
         * @returns `null` if valid, otherwise the reason why it is not
         */
        public static verify(message: { [k: string]: any }): (string|null);

        /**
         * Creates an ActivitySummary message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns ActivitySummary
         */
        public static fromObject(object: { [k: string]: any }): lifelog.ActivitySummary;

        /**
         * Creates a plain object from an ActivitySummary message. Also converts values to other types if specified.
         * @param message ActivitySummary
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: lifelog.ActivitySummary, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this ActivitySummary to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for ActivitySummary
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }

    /** Properties of an ActivityPeriod. */
    interface IActivityPeriod {

        /** ActivityPeriod startTime */
        startTime?: (string|null);

        /** ActivityPeriod endTime */
        endTime?: (string|null);

        /** ActivityPeriod primaryActivity */
        primaryActivity?: (string|null);

        /** ActivityPeriod appsUsed */
        appsUsed?: ({ [k: string]: number }|null);

        /** ActivityPeriod activityLevel */
        activityLevel?: (number|null);
    }

    /** Represents an ActivityPeriod. */
    class ActivityPeriod implements IActivityPeriod {

        /**
         * Constructs a new ActivityPeriod.
         * @param [properties] Properties to set
         */
        constructor(properties?: lifelog.IActivityPeriod);

        /** ActivityPeriod startTime. */
        public startTime: string;

        /** ActivityPeriod endTime. */
        public endTime: string;

        /** ActivityPeriod primaryActivity. */
        public primaryActivity: string;

        /** ActivityPeriod appsUsed. */
        public appsUsed: { [k: string]: number };

        /** ActivityPeriod activityLevel. */
        public activityLevel: number;

        /**
         * Creates a new ActivityPeriod instance using the specified properties.
         * @param [properties] Properties to set
         * @returns ActivityPeriod instance
         */
        public static create(properties?: lifelog.IActivityPeriod): lifelog.ActivityPeriod;

        /**
         * Encodes the specified ActivityPeriod message. Does not implicitly {@link lifelog.ActivityPeriod.verify|verify} messages.
         * @param message ActivityPeriod message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: lifelog.IActivityPeriod, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified ActivityPeriod message, length delimited. Does not implicitly {@link lifelog.ActivityPeriod.verify|verify} messages.
         * @param message ActivityPeriod message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: lifelog.IActivityPeriod, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes an ActivityPeriod message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns ActivityPeriod
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): lifelog.ActivityPeriod;

        /**
         * Decodes an ActivityPeriod message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns ActivityPeriod
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): lifelog.ActivityPeriod;

        /**
         * Verifies an ActivityPeriod message.
         * @param message Plain object to verify
         * @returns `null` if valid, otherwise the reason why it is not
         */
        public static verify(message: { [k: string]: any }): (string|null);

        /**
         * Creates an ActivityPeriod message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns ActivityPeriod
         */
        public static fromObject(object: { [k: string]: any }): lifelog.ActivityPeriod;

        /**
         * Creates a plain object from an ActivityPeriod message. Also converts values to other types if specified.
         * @param message ActivityPeriod
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: lifelog.ActivityPeriod, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this ActivityPeriod to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for ActivityPeriod
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }

    /** Properties of a LoginRequest. */
    interface ILoginRequest {

        /** LoginRequest username */
        username?: (string|null);

        /** LoginRequest password */
        password?: (string|null);
    }

    /** Represents a LoginRequest. */
    class LoginRequest implements ILoginRequest {

        /**
         * Constructs a new LoginRequest.
         * @param [properties] Properties to set
         */
        constructor(properties?: lifelog.ILoginRequest);

        /** LoginRequest username. */
        public username: string;

        /** LoginRequest password. */
        public password: string;

        /**
         * Creates a new LoginRequest instance using the specified properties.
         * @param [properties] Properties to set
         * @returns LoginRequest instance
         */
        public static create(properties?: lifelog.ILoginRequest): lifelog.LoginRequest;

        /**
         * Encodes the specified LoginRequest message. Does not implicitly {@link lifelog.LoginRequest.verify|verify} messages.
         * @param message LoginRequest message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: lifelog.ILoginRequest, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified LoginRequest message, length delimited. Does not implicitly {@link lifelog.LoginRequest.verify|verify} messages.
         * @param message LoginRequest message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: lifelog.ILoginRequest, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes a LoginRequest message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns LoginRequest
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): lifelog.LoginRequest;

        /**
         * Decodes a LoginRequest message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns LoginRequest
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): lifelog.LoginRequest;

        /**
         * Verifies a LoginRequest message.
         * @param message Plain object to verify
         * @returns `null` if valid, otherwise the reason why it is not
         */
        public static verify(message: { [k: string]: any }): (string|null);

        /**
         * Creates a LoginRequest message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns LoginRequest
         */
        public static fromObject(object: { [k: string]: any }): lifelog.LoginRequest;

        /**
         * Creates a plain object from a LoginRequest message. Also converts values to other types if specified.
         * @param message LoginRequest
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: lifelog.LoginRequest, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this LoginRequest to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for LoginRequest
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }

    /** Properties of a LoginResponse. */
    interface ILoginResponse {

        /** LoginResponse token */
        token?: (string|null);

        /** LoginResponse success */
        success?: (boolean|null);

        /** LoginResponse errorMessage */
        errorMessage?: (string|null);

        /** LoginResponse userProfile */
        userProfile?: (lifelog.IUserProfile|null);
    }

    /** Represents a LoginResponse. */
    class LoginResponse implements ILoginResponse {

        /**
         * Constructs a new LoginResponse.
         * @param [properties] Properties to set
         */
        constructor(properties?: lifelog.ILoginResponse);

        /** LoginResponse token. */
        public token: string;

        /** LoginResponse success. */
        public success: boolean;

        /** LoginResponse errorMessage. */
        public errorMessage: string;

        /** LoginResponse userProfile. */
        public userProfile?: (lifelog.IUserProfile|null);

        /**
         * Creates a new LoginResponse instance using the specified properties.
         * @param [properties] Properties to set
         * @returns LoginResponse instance
         */
        public static create(properties?: lifelog.ILoginResponse): lifelog.LoginResponse;

        /**
         * Encodes the specified LoginResponse message. Does not implicitly {@link lifelog.LoginResponse.verify|verify} messages.
         * @param message LoginResponse message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: lifelog.ILoginResponse, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified LoginResponse message, length delimited. Does not implicitly {@link lifelog.LoginResponse.verify|verify} messages.
         * @param message LoginResponse message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: lifelog.ILoginResponse, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes a LoginResponse message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns LoginResponse
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): lifelog.LoginResponse;

        /**
         * Decodes a LoginResponse message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns LoginResponse
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): lifelog.LoginResponse;

        /**
         * Verifies a LoginResponse message.
         * @param message Plain object to verify
         * @returns `null` if valid, otherwise the reason why it is not
         */
        public static verify(message: { [k: string]: any }): (string|null);

        /**
         * Creates a LoginResponse message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns LoginResponse
         */
        public static fromObject(object: { [k: string]: any }): lifelog.LoginResponse;

        /**
         * Creates a plain object from a LoginResponse message. Also converts values to other types if specified.
         * @param message LoginResponse
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: lifelog.LoginResponse, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this LoginResponse to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for LoginResponse
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }

    /** Properties of a RegisterRequest. */
    interface IRegisterRequest {

        /** RegisterRequest username */
        username?: (string|null);

        /** RegisterRequest password */
        password?: (string|null);

        /** RegisterRequest email */
        email?: (string|null);

        /** RegisterRequest displayName */
        displayName?: (string|null);
    }

    /** Represents a RegisterRequest. */
    class RegisterRequest implements IRegisterRequest {

        /**
         * Constructs a new RegisterRequest.
         * @param [properties] Properties to set
         */
        constructor(properties?: lifelog.IRegisterRequest);

        /** RegisterRequest username. */
        public username: string;

        /** RegisterRequest password. */
        public password: string;

        /** RegisterRequest email. */
        public email: string;

        /** RegisterRequest displayName. */
        public displayName: string;

        /**
         * Creates a new RegisterRequest instance using the specified properties.
         * @param [properties] Properties to set
         * @returns RegisterRequest instance
         */
        public static create(properties?: lifelog.IRegisterRequest): lifelog.RegisterRequest;

        /**
         * Encodes the specified RegisterRequest message. Does not implicitly {@link lifelog.RegisterRequest.verify|verify} messages.
         * @param message RegisterRequest message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: lifelog.IRegisterRequest, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified RegisterRequest message, length delimited. Does not implicitly {@link lifelog.RegisterRequest.verify|verify} messages.
         * @param message RegisterRequest message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: lifelog.IRegisterRequest, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes a RegisterRequest message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns RegisterRequest
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): lifelog.RegisterRequest;

        /**
         * Decodes a RegisterRequest message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns RegisterRequest
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): lifelog.RegisterRequest;

        /**
         * Verifies a RegisterRequest message.
         * @param message Plain object to verify
         * @returns `null` if valid, otherwise the reason why it is not
         */
        public static verify(message: { [k: string]: any }): (string|null);

        /**
         * Creates a RegisterRequest message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns RegisterRequest
         */
        public static fromObject(object: { [k: string]: any }): lifelog.RegisterRequest;

        /**
         * Creates a plain object from a RegisterRequest message. Also converts values to other types if specified.
         * @param message RegisterRequest
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: lifelog.RegisterRequest, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this RegisterRequest to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for RegisterRequest
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }

    /** Properties of a RegisterResponse. */
    interface IRegisterResponse {

        /** RegisterResponse success */
        success?: (boolean|null);

        /** RegisterResponse errorMessage */
        errorMessage?: (string|null);

        /** RegisterResponse token */
        token?: (string|null);
    }

    /** Represents a RegisterResponse. */
    class RegisterResponse implements IRegisterResponse {

        /**
         * Constructs a new RegisterResponse.
         * @param [properties] Properties to set
         */
        constructor(properties?: lifelog.IRegisterResponse);

        /** RegisterResponse success. */
        public success: boolean;

        /** RegisterResponse errorMessage. */
        public errorMessage: string;

        /** RegisterResponse token. */
        public token: string;

        /**
         * Creates a new RegisterResponse instance using the specified properties.
         * @param [properties] Properties to set
         * @returns RegisterResponse instance
         */
        public static create(properties?: lifelog.IRegisterResponse): lifelog.RegisterResponse;

        /**
         * Encodes the specified RegisterResponse message. Does not implicitly {@link lifelog.RegisterResponse.verify|verify} messages.
         * @param message RegisterResponse message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: lifelog.IRegisterResponse, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified RegisterResponse message, length delimited. Does not implicitly {@link lifelog.RegisterResponse.verify|verify} messages.
         * @param message RegisterResponse message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: lifelog.IRegisterResponse, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes a RegisterResponse message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns RegisterResponse
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): lifelog.RegisterResponse;

        /**
         * Decodes a RegisterResponse message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns RegisterResponse
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): lifelog.RegisterResponse;

        /**
         * Verifies a RegisterResponse message.
         * @param message Plain object to verify
         * @returns `null` if valid, otherwise the reason why it is not
         */
        public static verify(message: { [k: string]: any }): (string|null);

        /**
         * Creates a RegisterResponse message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns RegisterResponse
         */
        public static fromObject(object: { [k: string]: any }): lifelog.RegisterResponse;

        /**
         * Creates a plain object from a RegisterResponse message. Also converts values to other types if specified.
         * @param message RegisterResponse
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: lifelog.RegisterResponse, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this RegisterResponse to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for RegisterResponse
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }

    /** Properties of a UserRequest. */
    interface IUserRequest {

        /** UserRequest userId */
        userId?: (string|null);
    }

    /** Represents a UserRequest. */
    class UserRequest implements IUserRequest {

        /**
         * Constructs a new UserRequest.
         * @param [properties] Properties to set
         */
        constructor(properties?: lifelog.IUserRequest);

        /** UserRequest userId. */
        public userId: string;

        /**
         * Creates a new UserRequest instance using the specified properties.
         * @param [properties] Properties to set
         * @returns UserRequest instance
         */
        public static create(properties?: lifelog.IUserRequest): lifelog.UserRequest;

        /**
         * Encodes the specified UserRequest message. Does not implicitly {@link lifelog.UserRequest.verify|verify} messages.
         * @param message UserRequest message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: lifelog.IUserRequest, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified UserRequest message, length delimited. Does not implicitly {@link lifelog.UserRequest.verify|verify} messages.
         * @param message UserRequest message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: lifelog.IUserRequest, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes a UserRequest message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns UserRequest
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): lifelog.UserRequest;

        /**
         * Decodes a UserRequest message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns UserRequest
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): lifelog.UserRequest;

        /**
         * Verifies a UserRequest message.
         * @param message Plain object to verify
         * @returns `null` if valid, otherwise the reason why it is not
         */
        public static verify(message: { [k: string]: any }): (string|null);

        /**
         * Creates a UserRequest message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns UserRequest
         */
        public static fromObject(object: { [k: string]: any }): lifelog.UserRequest;

        /**
         * Creates a plain object from a UserRequest message. Also converts values to other types if specified.
         * @param message UserRequest
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: lifelog.UserRequest, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this UserRequest to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for UserRequest
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }

    /** Properties of a UserProfile. */
    interface IUserProfile {

        /** UserProfile userId */
        userId?: (string|null);

        /** UserProfile username */
        username?: (string|null);

        /** UserProfile displayName */
        displayName?: (string|null);

        /** UserProfile email */
        email?: (string|null);

        /** UserProfile createdAt */
        createdAt?: (string|null);

        /** UserProfile settings */
        settings?: ({ [k: string]: boolean }|null);
    }

    /** Represents a UserProfile. */
    class UserProfile implements IUserProfile {

        /**
         * Constructs a new UserProfile.
         * @param [properties] Properties to set
         */
        constructor(properties?: lifelog.IUserProfile);

        /** UserProfile userId. */
        public userId: string;

        /** UserProfile username. */
        public username: string;

        /** UserProfile displayName. */
        public displayName: string;

        /** UserProfile email. */
        public email: string;

        /** UserProfile createdAt. */
        public createdAt: string;

        /** UserProfile settings. */
        public settings: { [k: string]: boolean };

        /**
         * Creates a new UserProfile instance using the specified properties.
         * @param [properties] Properties to set
         * @returns UserProfile instance
         */
        public static create(properties?: lifelog.IUserProfile): lifelog.UserProfile;

        /**
         * Encodes the specified UserProfile message. Does not implicitly {@link lifelog.UserProfile.verify|verify} messages.
         * @param message UserProfile message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: lifelog.IUserProfile, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified UserProfile message, length delimited. Does not implicitly {@link lifelog.UserProfile.verify|verify} messages.
         * @param message UserProfile message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: lifelog.IUserProfile, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes a UserProfile message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns UserProfile
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): lifelog.UserProfile;

        /**
         * Decodes a UserProfile message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns UserProfile
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): lifelog.UserProfile;

        /**
         * Verifies a UserProfile message.
         * @param message Plain object to verify
         * @returns `null` if valid, otherwise the reason why it is not
         */
        public static verify(message: { [k: string]: any }): (string|null);

        /**
         * Creates a UserProfile message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns UserProfile
         */
        public static fromObject(object: { [k: string]: any }): lifelog.UserProfile;

        /**
         * Creates a plain object from a UserProfile message. Also converts values to other types if specified.
         * @param message UserProfile
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: lifelog.UserProfile, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this UserProfile to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for UserProfile
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }

    /** Properties of a LoggerStatusRequest. */
    interface ILoggerStatusRequest {

        /** LoggerStatusRequest loggerNames */
        loggerNames?: (string[]|null);
    }

    /** Represents a LoggerStatusRequest. */
    class LoggerStatusRequest implements ILoggerStatusRequest {

        /**
         * Constructs a new LoggerStatusRequest.
         * @param [properties] Properties to set
         */
        constructor(properties?: lifelog.ILoggerStatusRequest);

        /** LoggerStatusRequest loggerNames. */
        public loggerNames: string[];

        /**
         * Creates a new LoggerStatusRequest instance using the specified properties.
         * @param [properties] Properties to set
         * @returns LoggerStatusRequest instance
         */
        public static create(properties?: lifelog.ILoggerStatusRequest): lifelog.LoggerStatusRequest;

        /**
         * Encodes the specified LoggerStatusRequest message. Does not implicitly {@link lifelog.LoggerStatusRequest.verify|verify} messages.
         * @param message LoggerStatusRequest message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: lifelog.ILoggerStatusRequest, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified LoggerStatusRequest message, length delimited. Does not implicitly {@link lifelog.LoggerStatusRequest.verify|verify} messages.
         * @param message LoggerStatusRequest message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: lifelog.ILoggerStatusRequest, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes a LoggerStatusRequest message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns LoggerStatusRequest
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): lifelog.LoggerStatusRequest;

        /**
         * Decodes a LoggerStatusRequest message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns LoggerStatusRequest
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): lifelog.LoggerStatusRequest;

        /**
         * Verifies a LoggerStatusRequest message.
         * @param message Plain object to verify
         * @returns `null` if valid, otherwise the reason why it is not
         */
        public static verify(message: { [k: string]: any }): (string|null);

        /**
         * Creates a LoggerStatusRequest message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns LoggerStatusRequest
         */
        public static fromObject(object: { [k: string]: any }): lifelog.LoggerStatusRequest;

        /**
         * Creates a plain object from a LoggerStatusRequest message. Also converts values to other types if specified.
         * @param message LoggerStatusRequest
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: lifelog.LoggerStatusRequest, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this LoggerStatusRequest to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for LoggerStatusRequest
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }

    /** Properties of a LoggerStatus. */
    interface ILoggerStatus {

        /** LoggerStatus name */
        name?: (string|null);

        /** LoggerStatus enabled */
        enabled?: (boolean|null);

        /** LoggerStatus running */
        running?: (boolean|null);

        /** LoggerStatus lastActive */
        lastActive?: (string|null);

        /** LoggerStatus dataPoints */
        dataPoints?: (number|Long|null);

        /** LoggerStatus error */
        error?: (string|null);
    }

    /** Represents a LoggerStatus. */
    class LoggerStatus implements ILoggerStatus {

        /**
         * Constructs a new LoggerStatus.
         * @param [properties] Properties to set
         */
        constructor(properties?: lifelog.ILoggerStatus);

        /** LoggerStatus name. */
        public name: string;

        /** LoggerStatus enabled. */
        public enabled: boolean;

        /** LoggerStatus running. */
        public running: boolean;

        /** LoggerStatus lastActive. */
        public lastActive: string;

        /** LoggerStatus dataPoints. */
        public dataPoints: (number|Long);

        /** LoggerStatus error. */
        public error: string;

        /**
         * Creates a new LoggerStatus instance using the specified properties.
         * @param [properties] Properties to set
         * @returns LoggerStatus instance
         */
        public static create(properties?: lifelog.ILoggerStatus): lifelog.LoggerStatus;

        /**
         * Encodes the specified LoggerStatus message. Does not implicitly {@link lifelog.LoggerStatus.verify|verify} messages.
         * @param message LoggerStatus message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: lifelog.ILoggerStatus, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified LoggerStatus message, length delimited. Does not implicitly {@link lifelog.LoggerStatus.verify|verify} messages.
         * @param message LoggerStatus message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: lifelog.ILoggerStatus, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes a LoggerStatus message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns LoggerStatus
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): lifelog.LoggerStatus;

        /**
         * Decodes a LoggerStatus message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns LoggerStatus
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): lifelog.LoggerStatus;

        /**
         * Verifies a LoggerStatus message.
         * @param message Plain object to verify
         * @returns `null` if valid, otherwise the reason why it is not
         */
        public static verify(message: { [k: string]: any }): (string|null);

        /**
         * Creates a LoggerStatus message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns LoggerStatus
         */
        public static fromObject(object: { [k: string]: any }): lifelog.LoggerStatus;

        /**
         * Creates a plain object from a LoggerStatus message. Also converts values to other types if specified.
         * @param message LoggerStatus
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: lifelog.LoggerStatus, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this LoggerStatus to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for LoggerStatus
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }

    /** Properties of a LoggerStatusResponse. */
    interface ILoggerStatusResponse {

        /** LoggerStatusResponse loggers */
        loggers?: (lifelog.ILoggerStatus[]|null);

        /** LoggerStatusResponse systemStats */
        systemStats?: ({ [k: string]: string }|null);
    }

    /** Represents a LoggerStatusResponse. */
    class LoggerStatusResponse implements ILoggerStatusResponse {

        /**
         * Constructs a new LoggerStatusResponse.
         * @param [properties] Properties to set
         */
        constructor(properties?: lifelog.ILoggerStatusResponse);

        /** LoggerStatusResponse loggers. */
        public loggers: lifelog.ILoggerStatus[];

        /** LoggerStatusResponse systemStats. */
        public systemStats: { [k: string]: string };

        /**
         * Creates a new LoggerStatusResponse instance using the specified properties.
         * @param [properties] Properties to set
         * @returns LoggerStatusResponse instance
         */
        public static create(properties?: lifelog.ILoggerStatusResponse): lifelog.LoggerStatusResponse;

        /**
         * Encodes the specified LoggerStatusResponse message. Does not implicitly {@link lifelog.LoggerStatusResponse.verify|verify} messages.
         * @param message LoggerStatusResponse message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: lifelog.ILoggerStatusResponse, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified LoggerStatusResponse message, length delimited. Does not implicitly {@link lifelog.LoggerStatusResponse.verify|verify} messages.
         * @param message LoggerStatusResponse message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: lifelog.ILoggerStatusResponse, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes a LoggerStatusResponse message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns LoggerStatusResponse
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): lifelog.LoggerStatusResponse;

        /**
         * Decodes a LoggerStatusResponse message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns LoggerStatusResponse
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): lifelog.LoggerStatusResponse;

        /**
         * Verifies a LoggerStatusResponse message.
         * @param message Plain object to verify
         * @returns `null` if valid, otherwise the reason why it is not
         */
        public static verify(message: { [k: string]: any }): (string|null);

        /**
         * Creates a LoggerStatusResponse message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns LoggerStatusResponse
         */
        public static fromObject(object: { [k: string]: any }): lifelog.LoggerStatusResponse;

        /**
         * Creates a plain object from a LoggerStatusResponse message. Also converts values to other types if specified.
         * @param message LoggerStatusResponse
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: lifelog.LoggerStatusResponse, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this LoggerStatusResponse to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for LoggerStatusResponse
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }

    /** Properties of a ToggleLoggerRequest. */
    interface IToggleLoggerRequest {

        /** ToggleLoggerRequest loggerName */
        loggerName?: (string|null);

        /** ToggleLoggerRequest enable */
        enable?: (boolean|null);
    }

    /** Represents a ToggleLoggerRequest. */
    class ToggleLoggerRequest implements IToggleLoggerRequest {

        /**
         * Constructs a new ToggleLoggerRequest.
         * @param [properties] Properties to set
         */
        constructor(properties?: lifelog.IToggleLoggerRequest);

        /** ToggleLoggerRequest loggerName. */
        public loggerName: string;

        /** ToggleLoggerRequest enable. */
        public enable: boolean;

        /**
         * Creates a new ToggleLoggerRequest instance using the specified properties.
         * @param [properties] Properties to set
         * @returns ToggleLoggerRequest instance
         */
        public static create(properties?: lifelog.IToggleLoggerRequest): lifelog.ToggleLoggerRequest;

        /**
         * Encodes the specified ToggleLoggerRequest message. Does not implicitly {@link lifelog.ToggleLoggerRequest.verify|verify} messages.
         * @param message ToggleLoggerRequest message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: lifelog.IToggleLoggerRequest, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified ToggleLoggerRequest message, length delimited. Does not implicitly {@link lifelog.ToggleLoggerRequest.verify|verify} messages.
         * @param message ToggleLoggerRequest message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: lifelog.IToggleLoggerRequest, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes a ToggleLoggerRequest message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns ToggleLoggerRequest
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): lifelog.ToggleLoggerRequest;

        /**
         * Decodes a ToggleLoggerRequest message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns ToggleLoggerRequest
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): lifelog.ToggleLoggerRequest;

        /**
         * Verifies a ToggleLoggerRequest message.
         * @param message Plain object to verify
         * @returns `null` if valid, otherwise the reason why it is not
         */
        public static verify(message: { [k: string]: any }): (string|null);

        /**
         * Creates a ToggleLoggerRequest message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns ToggleLoggerRequest
         */
        public static fromObject(object: { [k: string]: any }): lifelog.ToggleLoggerRequest;

        /**
         * Creates a plain object from a ToggleLoggerRequest message. Also converts values to other types if specified.
         * @param message ToggleLoggerRequest
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: lifelog.ToggleLoggerRequest, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this ToggleLoggerRequest to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for ToggleLoggerRequest
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }

    /** Properties of a ToggleLoggerResponse. */
    interface IToggleLoggerResponse {

        /** ToggleLoggerResponse success */
        success?: (boolean|null);

        /** ToggleLoggerResponse errorMessage */
        errorMessage?: (string|null);

        /** ToggleLoggerResponse status */
        status?: (lifelog.ILoggerStatus|null);
    }

    /** Represents a ToggleLoggerResponse. */
    class ToggleLoggerResponse implements IToggleLoggerResponse {

        /**
         * Constructs a new ToggleLoggerResponse.
         * @param [properties] Properties to set
         */
        constructor(properties?: lifelog.IToggleLoggerResponse);

        /** ToggleLoggerResponse success. */
        public success: boolean;

        /** ToggleLoggerResponse errorMessage. */
        public errorMessage: string;

        /** ToggleLoggerResponse status. */
        public status?: (lifelog.ILoggerStatus|null);

        /**
         * Creates a new ToggleLoggerResponse instance using the specified properties.
         * @param [properties] Properties to set
         * @returns ToggleLoggerResponse instance
         */
        public static create(properties?: lifelog.IToggleLoggerResponse): lifelog.ToggleLoggerResponse;

        /**
         * Encodes the specified ToggleLoggerResponse message. Does not implicitly {@link lifelog.ToggleLoggerResponse.verify|verify} messages.
         * @param message ToggleLoggerResponse message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: lifelog.IToggleLoggerResponse, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified ToggleLoggerResponse message, length delimited. Does not implicitly {@link lifelog.ToggleLoggerResponse.verify|verify} messages.
         * @param message ToggleLoggerResponse message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: lifelog.IToggleLoggerResponse, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes a ToggleLoggerResponse message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns ToggleLoggerResponse
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): lifelog.ToggleLoggerResponse;

        /**
         * Decodes a ToggleLoggerResponse message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns ToggleLoggerResponse
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): lifelog.ToggleLoggerResponse;

        /**
         * Verifies a ToggleLoggerResponse message.
         * @param message Plain object to verify
         * @returns `null` if valid, otherwise the reason why it is not
         */
        public static verify(message: { [k: string]: any }): (string|null);

        /**
         * Creates a ToggleLoggerResponse message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns ToggleLoggerResponse
         */
        public static fromObject(object: { [k: string]: any }): lifelog.ToggleLoggerResponse;

        /**
         * Creates a plain object from a ToggleLoggerResponse message. Also converts values to other types if specified.
         * @param message ToggleLoggerResponse
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: lifelog.ToggleLoggerResponse, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this ToggleLoggerResponse to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for ToggleLoggerResponse
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }

    /** Properties of a SnapshotRequest. */
    interface ISnapshotRequest {

        /** SnapshotRequest loggers */
        loggers?: (string[]|null);

        /** SnapshotRequest options */
        options?: ({ [k: string]: string }|null);
    }

    /** Represents a SnapshotRequest. */
    class SnapshotRequest implements ISnapshotRequest {

        /**
         * Constructs a new SnapshotRequest.
         * @param [properties] Properties to set
         */
        constructor(properties?: lifelog.ISnapshotRequest);

        /** SnapshotRequest loggers. */
        public loggers: string[];

        /** SnapshotRequest options. */
        public options: { [k: string]: string };

        /**
         * Creates a new SnapshotRequest instance using the specified properties.
         * @param [properties] Properties to set
         * @returns SnapshotRequest instance
         */
        public static create(properties?: lifelog.ISnapshotRequest): lifelog.SnapshotRequest;

        /**
         * Encodes the specified SnapshotRequest message. Does not implicitly {@link lifelog.SnapshotRequest.verify|verify} messages.
         * @param message SnapshotRequest message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: lifelog.ISnapshotRequest, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified SnapshotRequest message, length delimited. Does not implicitly {@link lifelog.SnapshotRequest.verify|verify} messages.
         * @param message SnapshotRequest message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: lifelog.ISnapshotRequest, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes a SnapshotRequest message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns SnapshotRequest
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): lifelog.SnapshotRequest;

        /**
         * Decodes a SnapshotRequest message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns SnapshotRequest
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): lifelog.SnapshotRequest;

        /**
         * Verifies a SnapshotRequest message.
         * @param message Plain object to verify
         * @returns `null` if valid, otherwise the reason why it is not
         */
        public static verify(message: { [k: string]: any }): (string|null);

        /**
         * Creates a SnapshotRequest message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns SnapshotRequest
         */
        public static fromObject(object: { [k: string]: any }): lifelog.SnapshotRequest;

        /**
         * Creates a plain object from a SnapshotRequest message. Also converts values to other types if specified.
         * @param message SnapshotRequest
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: lifelog.SnapshotRequest, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this SnapshotRequest to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for SnapshotRequest
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }

    /** Properties of a SnapshotResponse. */
    interface ISnapshotResponse {

        /** SnapshotResponse snapshotId */
        snapshotId?: (string|null);

        /** SnapshotResponse success */
        success?: (boolean|null);

        /** SnapshotResponse errorMessage */
        errorMessage?: (string|null);

        /** SnapshotResponse triggeredLoggers */
        triggeredLoggers?: (string[]|null);
    }

    /** Represents a SnapshotResponse. */
    class SnapshotResponse implements ISnapshotResponse {

        /**
         * Constructs a new SnapshotResponse.
         * @param [properties] Properties to set
         */
        constructor(properties?: lifelog.ISnapshotResponse);

        /** SnapshotResponse snapshotId. */
        public snapshotId: string;

        /** SnapshotResponse success. */
        public success: boolean;

        /** SnapshotResponse errorMessage. */
        public errorMessage: string;

        /** SnapshotResponse triggeredLoggers. */
        public triggeredLoggers: string[];

        /**
         * Creates a new SnapshotResponse instance using the specified properties.
         * @param [properties] Properties to set
         * @returns SnapshotResponse instance
         */
        public static create(properties?: lifelog.ISnapshotResponse): lifelog.SnapshotResponse;

        /**
         * Encodes the specified SnapshotResponse message. Does not implicitly {@link lifelog.SnapshotResponse.verify|verify} messages.
         * @param message SnapshotResponse message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: lifelog.ISnapshotResponse, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified SnapshotResponse message, length delimited. Does not implicitly {@link lifelog.SnapshotResponse.verify|verify} messages.
         * @param message SnapshotResponse message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: lifelog.ISnapshotResponse, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes a SnapshotResponse message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns SnapshotResponse
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): lifelog.SnapshotResponse;

        /**
         * Decodes a SnapshotResponse message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns SnapshotResponse
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): lifelog.SnapshotResponse;

        /**
         * Verifies a SnapshotResponse message.
         * @param message Plain object to verify
         * @returns `null` if valid, otherwise the reason why it is not
         */
        public static verify(message: { [k: string]: any }): (string|null);

        /**
         * Creates a SnapshotResponse message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns SnapshotResponse
         */
        public static fromObject(object: { [k: string]: any }): lifelog.SnapshotResponse;

        /**
         * Creates a plain object from a SnapshotResponse message. Also converts values to other types if specified.
         * @param message SnapshotResponse
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: lifelog.SnapshotResponse, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this SnapshotResponse to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for SnapshotResponse
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }
}
