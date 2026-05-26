import { handleWriteCreateRequest } from "./write-create";
import { handleWriteModifyRequest } from "./write-modify";
import { handleWriteStyleRequest } from "./write-styles";
import { handleWriteVariableRequest } from "./write-variables";
import { handleWriteComponentRequest } from "./write-components";
import { handleWritePrototypeRequest } from "./write-prototype";
import { handleWritePageRequest } from "./write-page";

export const handleWriteRequest = async (request: any) =>
  (await handleWriteCreateRequest(request)) ??
  (await handleWriteModifyRequest(request)) ??
  (await handleWriteStyleRequest(request)) ??
  (await handleWriteVariableRequest(request)) ??
  (await handleWriteComponentRequest(request)) ??
  (await handleWritePrototypeRequest(request)) ??
  (await handleWritePageRequest(request));
