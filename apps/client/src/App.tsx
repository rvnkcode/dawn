import { InboxOutlined, PlusOutlined } from "@ant-design/icons";
import { Button, Checkbox, Form, Input, Layout, List, Typography } from "antd";
import styled from "styled-components";

// CSS
const Header = styled.header`
  position: sticky;
  top: 0;
  z-index: 1;
  width: 95%;
  background: white;
  padding-top: 1rem;
  margin-bottom: 1.5rem;
`;

const { Title } = Typography;
const { Content, Footer } = Layout;

function App() {
  const [form] = Form.useForm();
  const list: string[] = [];

  return (
    <Layout
      style={{
        background: "none",
        margin: "auto",
        width: "95%",
        maxWidth: "960px"
      }}
    >
      <Header style={{ width: "100%" }}>
        <Title style={{ fontSize: "1.8rem" }}>
          <InboxOutlined style={{ color: "#1677FF" }} />
          Inbox
        </Title>
        <Form layout="inline" form={form}>
          <Input.Group compact>
            <Form.Item name="title" noStyle rules={[{ required: true }]}>
              <Input style={{ width: "calc(100% - 46px)" }} autoFocus placeholder="New To-Do" />
            </Form.Item>
            <Button type="primary" htmlType="submit">
              <PlusOutlined />
            </Button>
          </Input.Group>
        </Form>
      </Header>
      <Content>
        <List
          dataSource={list}
          size="small"
          split={false}
          renderItem={(task) => (
            <List.Item style={{ padding: "0", marginBottom: "0.25rem" }}>
              <Checkbox>{task}</Checkbox>
            </List.Item>
          )}
        ></List>
      </Content>
    </Layout>
  );
}

export default App;
