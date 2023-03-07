import { CreateTaskDto, TaskEntity } from "../Api";
import PageTitleComponent from "./Components/PageTitleComponent";
import TaskEditModal from "./Components/TaskEditModal";
import { api, listState, selectedTaskIdState, isModalOpenState, modalInputValueState } from "./selectors/task";
import "./style.css";
import { PlusOutlined, DeleteFilled } from "@ant-design/icons";
import { Button, Checkbox, Empty, Form, Input, Layout, message } from "antd";
import { CheckboxChangeEvent } from "antd/es/checkbox";
import { MouseEvent } from "react";
import { useRecoilState, useSetRecoilState } from "recoil";
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

const { Content, Footer } = Layout;

function App() {
  const [form] = Form.useForm();
  const [list, setList] = useRecoilState(listState);

  // POST
  const createTask = (value: CreateTaskDto) => {
    api.task
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
      api.task
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
  const deleteSelectedTask = (e: MouseEvent<HTMLAnchorElement> | MouseEvent<HTMLButtonElement>) => {
    api.task
      .appControllerDeleteTask((e.currentTarget as HTMLButtonElement).value)
      .then((response) => {
        setList(list.filter((task) => task.id !== response.data.id));
      })
      .catch((error) => {
        console.error(error);
        message.error(`Cannot remove selected task`);
      });
  };

  const toggleChecked = (e: CheckboxChangeEvent) => {
    api.task.appControllerUpdateTask(e.target.value, { isDone: e.target.checked }).catch((error) => {
      console.error(error);
      message.error(`Cannot complete task`);
    });
  };

  // Task edit modal
  const setIsModalOpen = useSetRecoilState(isModalOpenState);
  const setModalInputValue = useSetRecoilState(modalInputValueState);
  const setSelectedTaskIdState = useSetRecoilState(selectedTaskIdState);

  const openTaskEditModal = (e: MouseEvent<HTMLAnchorElement> | MouseEvent<HTMLButtonElement>) => {
    setModalInputValue(e.currentTarget.innerText);
    setSelectedTaskIdState((e.currentTarget as HTMLButtonElement).value);
    setIsModalOpen(true);
  };

  const todoListComponent = (list: Array<TaskEntity>) => {
    const listItems = list.map((task) => (
      <li key={task.id} style={{ display: "flex", justifyContent: "space-between" }}>
        <Checkbox defaultChecked={task.isDone} onChange={toggleChecked} value={task.id}>
          <Button type="text" size="small" onClick={(e) => openTaskEditModal(e)} value={task.id}>
            <span>{task.title}</span>
          </Button>
        </Checkbox>
        <Button type="text" size="small" value={task.id} onClick={(e) => deleteSelectedTask(e)}>
          <DeleteFilled />
        </Button>
      </li>
    ));

    return <ul>{listItems}</ul>;
  };

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
        <PageTitleComponent />

        <Form onFinish={createTask} form={form}>
          <Input.Group compact>
            <Form.Item name="title" noStyle rules={[{ required: true }]}>
              <Input
                style={{ width: "calc(100% - 46px)" }}
                autoFocus
                placeholder="New To-Do"
                required
                autoComplete="off"
              />
            </Form.Item>
            <Button type="primary" htmlType="submit">
              <PlusOutlined />
            </Button>
          </Input.Group>
        </Form>
      </Header>

      <Content>
        {list.length > 0 ? (
          todoListComponent(list)
        ) : (
          <Empty image={Empty.PRESENTED_IMAGE_SIMPLE} description="Your inbox is empty - time to celebrate!" />
        )}
      </Content>

      {/* TODO: Add task counter and progress to footer */}
      <Footer style={{ position: "fixed", bottom: "0", left: "0", width: "100%", textAlign: "center" }}>
        <Button onClick={clearList} disabled={list.length == 0 ? true : false}>
          <DeleteFilled />
        </Button>
      </Footer>

      <TaskEditModal />
    </Layout>
  );
}

export default App;
