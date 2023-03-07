import App from "./App";
import "./reset.css";
import { LoadingOutlined } from "@ant-design/icons";
import { Spin } from "antd";
import React from "react";
import ReactDOM from "react-dom/client";
import { RecoilRoot } from "recoil";

const loadingIcon = <LoadingOutlined spin />;

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <RecoilRoot>
      <React.Suspense fallback={<Spin indicator={loadingIcon} />}>
        <App />
      </React.Suspense>
    </RecoilRoot>
  </React.StrictMode>
);
