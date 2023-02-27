import { Api, CreateTaskDto, TaskEntity, UpdateTaskDto } from "../Api";
import { InboxOutlined, PlusOutlined, DeleteFilled, EditFilled } from "@ant-design/icons";
import { Button, Checkbox, Form, Input, Layout, List, message, Modal, Typography } from "antd";
import { CheckboxChangeEvent } from "antd/es/checkbox";
import { MouseEvent, useEffect, useRef, useState } from "react";
import type { DraggableData, DraggableEvent } from "react-draggable";
import Draggable from "react-draggable";
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

// Locale
// const timeZone = Intl.DateTimeFormat().resolvedOptions().timeZone;

function App() {
  const [form] = Form.useForm();
  const [list, setList] = useState<TaskEntity[]>([]);

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
          message.error(`Cannot clear to-do list`);
        });
    }
  };

  const toggleChecked = (e: CheckboxChangeEvent) => {
    // message.info(`${e.target.value}`);
    api.id.appControllerUpdateTask(e.target.value, { isDone: e.target.checked }).catch((error) => {
      console.error(error);
      message.error(`Cannot complete task`);
    });
  };

  // Task edit modal
  const [isModalOpen, setIsModalOpen] = useState(false);
  const [modalInputValue, setModalInputValue] = useState(``);
  const [selectedTaskId, setSelectedTaskId] = useState(``);
  // Draggable modal states
  const draggableRef = useRef<HTMLDivElement>(null);
  const [bounds, setBounds] = useState({ left: 0, top: 0, bottom: 0, right: 0 });
  const [isDragEnabled, setIsDragEnabled] = useState(false);

  const openTaskEditModal = (e: MouseEvent<HTMLAnchorElement> | MouseEvent<HTMLButtonElement>) => {
    setModalInputValue(e.currentTarget.innerText);
    setSelectedTaskId((e.currentTarget as HTMLButtonElement).value);
    setIsModalOpen(true);
  };

  //Draggable modal
  const onModalStart = (_event: DraggableEvent, uiData: DraggableData) => {
    const { clientWidth, clientHeight } = window.document.documentElement;
    const targetRect = draggableRef.current?.getBoundingClientRect();
    if (!targetRect) {
      return;
    }

    setBounds({
      left: -targetRect.left + uiData.x,
      right: clientWidth - (targetRect.right - uiData.x),
      top: -targetRect.top + uiData.y,
      bottom: clientHeight - (targetRect.bottom - uiData.y)
    });
  };

  const editTask = (value: UpdateTaskDto) => {
    api.id
      .appControllerUpdateTask(selectedTaskId, value)
      .then((response) => {
        const updateList = [...list];
        updateList[updateList.findIndex((task) => task.id === +selectedTaskId)] = response.data;
        setList(updateList);
        setIsModalOpen(false);
      })
      .catch((error) => {
        console.error(error);
        message.error(`Cannot update task`);
      });
  };

  const deleteSelectedTask = (e: MouseEvent<HTMLAnchorElement> | MouseEvent<HTMLButtonElement>) => {
    api.id
      .appControllerDeleteTask((e.currentTarget as HTMLButtonElement).value)
      .then((response) => {
        setList(list.filter((task) => task.id !== response.data.id));
      })
      .catch((error) => {
        console.error(error);
        message.error(`Cannot remove selected task`);
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
        message.error(`Cannot get to-do list`);
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
      {/* TODO: Add today's date in header */}
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
            <List.Item style={{ padding: "0", marginBottom: "0.25rem" }}>
              <Checkbox defaultChecked={task.isDone} onChange={toggleChecked} value={task.id}>
                {/* TODO: Conditional rendering between button and span or change button status */}
                <Button type="text" size="small" onClick={(e) => openTaskEditModal(e)} value={task.id}>
                  {task.title}
                </Button>
              </Checkbox>
              <Button type="text" size="small" value={task.id} onClick={(e) => deleteSelectedTask(e)}>
                <DeleteFilled />
              </Button>
            </List.Item>
          )}
        ></List>
      </Content>
      {/* TODO: Add task counter and progress to footer */}
      <Footer style={{ position: "fixed", bottom: "0", left: "0", width: "100%", textAlign: "center" }}>
        <Button onClick={clearList}>
          <DeleteFilled />
        </Button>
      </Footer>

      {/* Editor modal component */}
      <Modal
        title={
          <div
            onMouseOver={() => {
              if (!isDragEnabled) {
                setIsDragEnabled(true);
              }
            }}
            onMouseOut={() => {
              setIsDragEnabled(false);
            }}
          >
            Edit To-Do
          </div>
        }
        open={isModalOpen}
        footer={null}
        onCancel={() => {
          setIsModalOpen(false);
        }}
        closable={false}
        destroyOnClose={true}
        modalRender={(modal) => (
          <Draggable disabled={isDragEnabled} bounds={bounds} onStart={(event, uiData) => onModalStart(event, uiData)}>
            <div ref={draggableRef}>{modal}</div>
          </Draggable>
        )}
      >
        <Form
          onFinish={editTask}
          style={{
            cursor: "move"
          }}
        >
          <Form.Item name="title" initialValue={modalInputValue}>
            <Input required placeholder="To-Do" />
          </Form.Item>
          <div style={{ textAlign: "right" }}>
            <Button htmlType="submit" type="primary">
              <EditFilled />
            </Button>
          </div>
        </Form>
      </Modal>
    </Layout>
  );
}

export default App;
