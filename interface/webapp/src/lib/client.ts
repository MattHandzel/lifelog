import { createPromiseClient } from "@connectrpc/connect";
import { createGrpcWebTransport } from "@connectrpc/connect-web";
import { LifelogServerService } from "../gen/lifelog_connect";

const transport = createGrpcWebTransport({
  baseUrl: window.location.origin, 
  interceptors: [
    (next) => async (req) => {
      const token = localStorage.getItem("LIFELOG_AUTH_TOKEN") || "a8e8b1dd3a2c4f31b97ae82e3879ea05";
      req.header.set("Authorization", `Bearer ${token}`);
      return next(req);
    },
  ],
});

export const client = createPromiseClient(LifelogServerService, transport);
