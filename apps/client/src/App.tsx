import { Api, CreateTaskDto, Task } from "../Api";
import { InboxOutlined, PlusOutlined, DeleteFilled, EditOutlined } from "@ant-design/icons";
import { Button, Checkbox, Form, Input, Layout, List, message, Modal, Typography } from "antd";
import { CheckboxChangeEvent } from "antd/es/checkbox";
import { useEffect, useState } from "react";
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

// API
const api = new Api({
  baseUrl: "http://localhost:3000"
});

function App() {
  const [form] = Form.useForm();
  const [list, setList] = useState<Task[]>([]);

  // POST
  const createTask = (value: CreateTaskDto) => {
    api
      .appControllerCreateTask(value)
      .then((response) => {
        setList(list.concat(response.data));
        form.resetFields();
      })
      .catch((error) => {
        console.error(error);
        message.error(`${error}`);
      });
  };

  // DELETE
  const clearList = () => {
    if (list.length > 0) {
      api
        .appControllerDeleteAllTask()
        .then(() => {
          setList([]);
        })
        .catch((error) => {
          console.error(error);
          message.error(`${error}`);
        });
    }
  };

  const toggleChecked = (e: CheckboxChangeEvent) => {
    // message.info(`${e.target.value}`);
    api.id
      .appControllerUpdateTask(e.target.value, { isDone: e.target.checked })
      .then(() => {})
      .catch((error) => {
        console.error(error);
        message.error(`${error}`);
      });
  };

  // Task edit modal
  const [isModalOpen, setIsModalOpen] = useState(false);
  const [modalInputValue, setModalInputValue] = useState(``);
  const openTaskEditModal = (e: any) => {
    // message.info(`${e.currentTarget.value}`);
    setModalInputValue(e.currentTarget.value);
    setIsModalOpen(true);
  };
  const edit = () => {
    console.log();
  };
  const { confirm } = Modal;

  const showConfirm = () => {
    confirm({
      title: "Do you Want to delete these items?",
      content: "Some descriptions",
      onOk() {
        console.log("OK");
      },
      onCancel() {
        console.log("Cancel");
      }
    });
  };

  // GET
  useEffect(() => {
    api
      .appControllerGetTaskList()
      .then((response) => {
        setList(response.data);
      })
      .catch((error) => {
        console.error(error);
        message.error(`${error}`);
      });
  }, []);

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
        <Form onFinish={createTask} form={form}>
          <Input.Group compact>
            <Form.Item name="title" noStyle rules={[{ required: true }]}>
              <Input style={{ width: "calc(100% - 46px)" }} autoFocus placeholder="New To-Do" required />
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
            <List.Item style={{ padding: "0", marginBottom: "0.25rem", justifyContent: "normal" }}>
              <Checkbox defaultChecked={task.isDone} onChange={toggleChecked} value={task.id} />
              {/* Conditional rendering between button and span */}
              <Button type="text" size="small" onClick={openTaskEditModal} value={task.title}>
                {task.title}
              </Button>
              <Button onClick={showConfirm} style={{ border: "none" }} size="small" value={task.title}>
                <EditOutlined />
              </Button>
            </List.Item>
          )}
        ></List>
      </Content>
      <Footer style={{ position: "fixed", bottom: "0", left: "0", width: "100%", textAlign: "center" }}>
        <Button onClick={clearList}>
          <DeleteFilled />
        </Button>
      </Footer>
      <Modal
        open={isModalOpen}
        footer={null}
        onCancel={() => {
          setIsModalOpen(false);
        }}
        closable={false}
        destroyOnClose={true}
        centered
        onOk={edit}
      >
        <Form>
          <Form.Item name="title" noStyle initialValue={modalInputValue}>
            <Input />
          </Form.Item>
          <Form.Item name="id" noStyle initialValue={`test`}>
            <input type="hidden" />
          </Form.Item>
        </Form>
      </Modal>
    </Layout>
  );
}

export default App;
