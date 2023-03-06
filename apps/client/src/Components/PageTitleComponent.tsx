import { InboxOutlined } from "@ant-design/icons";
import { Typography } from "antd";
import { format } from "date-fns-tz";

const { Title } = Typography;

// Locale
const timeZone = Intl.DateTimeFormat().resolvedOptions().timeZone;
const today = format(new Date(), "YYY.M.d eee", { timeZone: timeZone });
const week = format(new Date(), " (Io)", { timeZone: timeZone });

export default function TitleComponent() {
  return (
    <>
      <Title style={{ fontSize: "1.8rem", marginBottom: "0.25rem" }}>
        <InboxOutlined style={{ color: "#1677FF" }} />
        Inbox
      </Title>
      <h2 style={{ marginBottom: "1rem", fontSize: "1rem", color: "grey", fontWeight: "lighter" }}>
        {today}
        <span style={{ fontSize: "0.8rem" }}>{week}</span>
      </h2>
    </>
  );
}
